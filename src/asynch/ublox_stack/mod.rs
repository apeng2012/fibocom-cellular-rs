#[cfg(feature = "socket-tcp")]
pub mod tcp;
// #[cfg(feature = "socket-udp")]
// pub mod udp;

pub mod dns;

use core::cell::RefCell;
use core::fmt::Write;
use core::future::poll_fn;
use core::ops::{DerefMut, Rem};
use core::task::Poll;

use crate::asynch::state::Device;
use crate::command::Urc;

use self::dns::{DnsSocket, DnsState, DnsTable};

use super::state::{self, LinkState};
use super::AtHandle;

use atat::asynch::AtatClient;
use embassy_futures::select::{select4, Either4};
use embassy_sync::waitqueue::WakerRegistration;
use embassy_time::{Duration, Ticker};
use embedded_nal_async::SocketAddr;
use futures::pin_mut;
use heapless::{String, Vec};
use no_std_net::IpAddr;
use portable_atomic::{AtomicBool, AtomicU8, Ordering};
use ublox_sockets::{PeerHandle, Socket, SocketHandle, SocketSet, SocketStorage};

#[cfg(feature = "socket-tcp")]
use ublox_sockets::TcpState;
#[cfg(feature = "socket-udp")]
use ublox_sockets::UdpState;

const MAX_EGRESS_SIZE: usize = crate::command::ip_transport_layer::WRITE_DATA_MAX_LEN;

pub struct StackResources<const SOCK: usize> {
    sockets: [SocketStorage<'static>; SOCK],
}

impl<const SOCK: usize> StackResources<SOCK> {
    pub fn new() -> Self {
        Self {
            sockets: [SocketStorage::EMPTY; SOCK],
        }
    }
}

pub struct UbloxStack<AT: AtatClient + 'static, const URC_CAPACITY: usize> {
    socket: RefCell<SocketStack>,
    device: RefCell<state::Device<'static, AT, URC_CAPACITY>>,
    last_tx_socket: AtomicU8,
    should_tx: AtomicBool,
    link_up: AtomicBool,
}

struct SocketStack {
    sockets: SocketSet<'static>,
    waker: WakerRegistration,
    dns_table: DnsTable,
    dropped_sockets: heapless::Vec<PeerHandle, 3>,
    can_socket_be_opened: [Option<bool>; 6],
}

impl<AT: AtatClient + 'static, const URC_CAPACITY: usize> UbloxStack<AT, URC_CAPACITY> {
    pub fn new<const SOCK: usize>(
        device: state::Device<'static, AT, URC_CAPACITY>,
        resources: &'static mut StackResources<SOCK>,
    ) -> Self {
        let sockets = SocketSet::new(&mut resources.sockets[..]);

        let socket = SocketStack {
            sockets,
            dns_table: DnsTable::new(),
            waker: WakerRegistration::new(),
            dropped_sockets: heapless::Vec::new(),
            can_socket_be_opened: [None; 6],
        };

        Self {
            socket: RefCell::new(socket),
            device: RefCell::new(device),
            last_tx_socket: AtomicU8::new(0),
            link_up: AtomicBool::new(false),
            should_tx: AtomicBool::new(false),
        }
    }

    pub async fn run(&self) -> ! {
        let mut tx_buf = [0u8; MAX_EGRESS_SIZE];

        loop {
            // FIXME: It feels like this can be written smarter/simpler?
            let should_tx = poll_fn(|cx| match self.should_tx.load(Ordering::Relaxed) {
                true => {
                    self.should_tx.store(false, Ordering::Relaxed);
                    Poll::Ready(())
                }
                false => {
                    self.should_tx.store(true, Ordering::Relaxed);
                    self.socket.borrow_mut().waker.register(cx.waker());
                    Poll::<()>::Pending
                }
            });

            let ticker = Ticker::every(Duration::from_millis(100));
            pin_mut!(ticker);

            let mut device = self.device.borrow_mut();
            let Device {
                ref mut urc_subscription,
                ref mut shared,
                ref mut at,
                ..
            } = device.deref_mut();

            match select4(
                urc_subscription.next_message_pure(),
                should_tx,
                ticker.next(),
                poll_fn(
                    |cx| match (self.link_up.load(Ordering::Relaxed), shared.link_state(cx)) {
                        (true, Some(LinkState::Down)) => Poll::Ready(LinkState::Down),
                        (false, Some(LinkState::Up)) => Poll::Ready(LinkState::Up),
                        _ => Poll::Pending,
                    },
                ),
            )
            .await
            {
                Either4::First(event) => {
                    Self::socket_rx(event, &self.socket);
                }
                Either4::Second(_) | Either4::Third(_) => {
                    if let Some(ev) = self.tx_event(&mut tx_buf) {
                        Self::socket_tx(ev, &self.socket, at).await;
                    }
                }
                Either4::Fourth(new_state) => {
                    // Update link up
                    let old_link_up = self.link_up.load(Ordering::Relaxed);
                    let new_link_up = new_state == LinkState::Up;
                    self.link_up.store(new_link_up, Ordering::Relaxed);

                    // Print when changed
                    if old_link_up != new_link_up {
                        info!("link_up = {:?}", new_link_up);
                    }
                }
            }
        }
    }

    /// Make a query for a given name and return the corresponding IP addresses.
    // #[cfg(feature = "dns")]
    pub async fn dns_query(
        &self,
        name: &str,
        addr_type: embedded_nal_async::AddrType,
    ) -> Result<IpAddr, dns::Error> {
        DnsSocket::new(self).query(name, addr_type).await
    }

    fn socket_rx(event: Urc, socket: &RefCell<SocketStack>) {
        match event {
            Urc::SocketClosed(sc) => {
                let handle = sc.id;
                let mut s = socket.borrow_mut();
                for (_handle, socket) in s.sockets.iter_mut() {
                    match socket {
                        #[cfg(feature = "socket-udp")]
                        Socket::Udp(udp) if udp.peer_handle == Some(handle) => {
                            udp.peer_handle = None;
                            udp.set_state(UdpState::Closed);
                            break;
                        }
                        #[cfg(feature = "socket-tcp")]
                        Socket::Tcp(tcp) if tcp.peer_handle == Some(handle) => {
                            tcp.peer_handle = None;
                            tcp.set_state(TcpState::TimeWait);
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Urc::SocketOpened(so) => {
                Self::connect_event(SocketHandle(so.id.0 - 1), socket);
            }
            Urc::CanSocketOpen(cso) => {
                let mut s = socket.borrow_mut();
                for (i, oo) in s.can_socket_be_opened.iter_mut().enumerate() {
                    *oo = Some(!cso.id_list.contains(&PeerHandle(i as u8 + 1)));
                }
            }
            Urc::SocketDataSentOver(sdso) => {
                if sdso.status != crate::command::ip_transport_layer::types::SendStatus::Success {
                    warn!("Socket {} is flowed off", sdso.id.0);
                    return;
                }
                let handle = sdso.id;
                let mut s = socket.borrow_mut();
                for (_handle, socket) in s.sockets.iter_mut() {
                    match socket {
                        #[cfg(feature = "socket-udp")]
                        Socket::Udp(udp) if udp.peer_handle == Some(handle) => {
                            udp.set_state(UdpState::Established);
                            break;
                        }
                        #[cfg(feature = "socket-tcp")]
                        Socket::Tcp(tcp) if tcp.peer_handle == Some(handle) => {
                            tcp.set_state(TcpState::Established);
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Urc::SocketReadData(sda) => {
                let handle = sda.id;
                let mut s = socket.borrow_mut();
                for (_handle, socket) in s.sockets.iter_mut() {
                    match socket {
                        #[cfg(feature = "socket-udp")]
                        Socket::Udp(udp) if udp.peer_handle == Some(handle) => {
                            let n = udp.rx_enqueue_slice(&sda.data);
                            if n < sda.data.len() {
                                error!(
                                    "[{}] UDP RX data overflow! Discarding {} bytes",
                                    udp.peer_handle,
                                    sda.data.len() - n
                                );
                            }
                            break;
                        }
                        #[cfg(feature = "socket-tcp")]
                        Socket::Tcp(tcp) if tcp.peer_handle == Some(handle) => {
                            let n = tcp.rx_enqueue_slice(&sda.data);
                            if n < sda.data.len() {
                                error!(
                                    "[{}] TCP RX data overflow! Discarding {} bytes",
                                    tcp.peer_handle,
                                    sda.data.len() - n
                                );
                            }
                            break;
                        }
                        _ => {}
                    }
                }
            }
            _ => (),
        }
    }

    fn tx_event<'data>(&self, buf: &'data mut [u8]) -> Option<TxEvent<'data>> {
        let mut s = self.socket.borrow_mut();
        for query in s.dns_table.table.iter_mut() {
            if let DnsState::New = query.state {
                query.state = DnsState::Pending;
                buf[..query.domain_name.len()].copy_from_slice(query.domain_name.as_bytes());
                return Some(TxEvent::Dns {
                    hostname: core::str::from_utf8(&buf[..query.domain_name.len()]).unwrap(),
                });
            }
        }

        // Handle delayed close-by-drop here
        if let Some(dropped_peer_handle) = s.dropped_sockets.pop() {
            warn!("Handling dropped socket {}", dropped_peer_handle);
            return Some(TxEvent::Close {
                peer_handle: dropped_peer_handle,
            });
        }

        // Make sure to give all sockets an even opportunity to TX
        let skip = self
            .last_tx_socket
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
                let next = v + 1;
                Some(next.rem(s.sockets.sockets.len() as u8))
            })
            .unwrap();

        let can_socket_be_opened = s.can_socket_be_opened;
        let mut reset_be_open: (bool, Option<TxEvent>) = (false, None);

        for (handle, socket) in s.sockets.iter_mut().skip(skip as usize) {
            match socket {
                #[cfg(feature = "socket-udp")]
                Socket::Udp(_udp) => todo!(),
                #[cfg(feature = "socket-tcp")]
                Socket::Tcp(tcp) => {
                    tcp.poll();

                    match tcp.state() {
                        TcpState::Closed => {
                            if let Some(addr) = tcp.remote_endpoint() {
                                let SocketHandle(i) = handle;
                                if i >= 6 {
                                    continue;
                                }
                                match can_socket_be_opened[i as usize] {
                                    Some(true) => {
                                        tcp.set_state(TcpState::Established);
                                        reset_be_open.0 = true;
                                        break;
                                    }
                                    Some(false) => {
                                        reset_be_open.0 = true;
                                        reset_be_open.1 = Some(TxEvent::Connect {
                                            socket_handle: handle,
                                            socket_addr: addr,
                                        });
                                        break;
                                    }
                                    None => {
                                        return Some(TxEvent::CanBeOpened {
                                            socket_handle: handle,
                                        });
                                    }
                                }
                            }
                        }
                        // We transmit data in all states where we may have data in the buffer,
                        // or the transmit half of the connection is still open.
                        TcpState::Established | TcpState::CloseWait | TcpState::LastAck => {
                            if let Some(peer_handle) = tcp.peer_handle {
                                return tcp.tx_dequeue(|payload| {
                                    let len = core::cmp::min(payload.len(), MAX_EGRESS_SIZE);
                                    let res = if len != 0 {
                                        buf[..len].copy_from_slice(&payload[..len]);
                                        Some(TxEvent::Send {
                                            peer_handle,
                                            data: &buf[..len],
                                        })
                                    } else {
                                        None
                                    };

                                    (len, res)
                                });
                            }
                        }
                        TcpState::FinWait1 => {
                            return Some(TxEvent::Close {
                                peer_handle: tcp.peer_handle.unwrap(),
                            });
                        }
                        TcpState::Listen => todo!(),
                        TcpState::SynReceived => todo!(),
                        _ => {}
                    };
                }
                _ => {}
            };
        }

        match reset_be_open {
            (true, ret) => {
                s.can_socket_be_opened = [None; 6];
                ret
            }
            _ => None,
        }
    }

    async fn socket_tx<'data>(
        ev: TxEvent<'data>,
        socket: &RefCell<SocketStack>,
        at: &mut AtHandle<'_, AT>,
    ) {
        match ev {
            TxEvent::CanBeOpened { socket_handle } => {
                match at
                    .send(&crate::command::ip_transport_layer::CanSocketOpen)
                    .await
                {
                    Ok(_) => {
                        let mut s = socket.borrow_mut();
                        let tcp = s
                            .sockets
                            .get_mut::<ublox_sockets::tcp::Socket>(socket_handle);
                        tcp.set_state(TcpState::SynSent);
                    }
                    Err(e) => error!("Failed to can be opened?! {}", e),
                }
            }
            TxEvent::Connect {
                socket_handle,
                socket_addr,
            } => {
                let peer_handle = PeerHandle(socket_handle.0 + 1);
                let mut s = String::new();
                write!(&mut s, "{}", socket_addr.ip()).ok();
                let cmd = crate::command::ip_transport_layer::ConnectSocket {
                    id: peer_handle,
                    port: None,
                    remote_addr: s,
                    remote_port: socket_addr.port(),
                    protocol: crate::command::ip_transport_layer::types::SocketProtocol::TCP,
                };
                match at.send(&cmd).await {
                    Ok(_) => {
                        let mut s = socket.borrow_mut();
                        let tcp = s
                            .sockets
                            .get_mut::<ublox_sockets::tcp::Socket>(socket_handle);
                        tcp.peer_handle = Some(peer_handle);
                        tcp.set_state(TcpState::SynSent);
                    }
                    Err(e) => {
                        error!("Failed to connect?! {}", e)
                    }
                }
            }
            TxEvent::Send { peer_handle, data } => {
                warn!("Sending {} bytes on {}", data.len(), peer_handle.0);
                if let Err(e) = at
                    .send(&crate::command::ip_transport_layer::WriteSocketData {
                        id: peer_handle,
                        lenth: data.len() as u16,
                    })
                    .await
                {
                    error!("Failed to send?! {}", e);
                    return;
                }

                if let Err(e) = at
                    .send(&crate::command::ip_transport_layer::WriteData { buf: data })
                    .await
                {
                    error!("Failed to send data?! {}", e);
                    return;
                }

                let mut s = socket.borrow_mut();
                for (_handle, socket) in s.sockets.iter_mut() {
                    match socket {
                        #[cfg(feature = "socket-udp")]
                        Socket::Udp(udp) if udp.peer_handle == Some(peer_handle) => {
                            break;
                        }
                        #[cfg(feature = "socket-tcp")]
                        Socket::Tcp(tcp) if tcp.peer_handle == Some(peer_handle) => {
                            // wait +MIPSEND urc
                            tcp.set_state(TcpState::SynSent);
                            break;
                        }
                        _ => {}
                    }
                }
            }
            TxEvent::Close { peer_handle } => {
                at.send(&crate::command::ip_transport_layer::CloseSocket { id: peer_handle })
                    .await
                    .ok();
            }
            TxEvent::Dns { hostname } => {
                match at
                    .send(&crate::command::dns::ResolveNameIp {
                        ip_domain_string: hostname,
                    })
                    .await
                {
                    Ok(rnr) => {
                        let mut s = socket.borrow_mut();
                        if let Some(query) = s.dns_table.get_mut(&hostname) {
                            if query.state == DnsState::Pending {
                                let mut vec: Vec<u8, 16> = Vec::new();
                                vec.extend_from_slice(rnr.ip_addr.as_slice()).unwrap();
                                if let Ok(s) = String::from_utf8(vec) {
                                    match s.parse::<IpAddr>() {
                                        Ok(ip_addr) => {
                                            query.state = DnsState::Resolved(ip_addr);
                                        }
                                        Err(_) => {
                                            query.state = DnsState::Error;
                                        }
                                    }
                                    query.waker.wake();
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to dns?! {}", e);
                        let mut s = socket.borrow_mut();
                        if let Some(query) = s.dns_table.get_mut(&hostname) {
                            match query.state {
                                DnsState::Pending => {
                                    query.state = DnsState::Error;
                                    query.waker.wake();
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }

    fn connect_event(handle: SocketHandle, socket: &RefCell<SocketStack>) {
        let mut s = socket.borrow_mut();
        for (h, socket) in s.sockets.iter_mut() {
            if handle == h {
                match socket {
                    #[cfg(feature = "socket-tcp")]
                    Socket::Tcp(tcp) => {
                        tcp.set_state(TcpState::Established);
                        break;
                    }
                    #[cfg(feature = "socket-udp")]
                    Socket::Udp(udp) => {
                        udp.set_state(UdpState::Established);
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
}

// TODO: This extra data clone step can probably be avoided by adding a
// waker/context based API to ATAT.
enum TxEvent<'data> {
    CanBeOpened {
        socket_handle: SocketHandle,
    },
    Connect {
        socket_handle: SocketHandle,
        socket_addr: SocketAddr,
    },
    Send {
        peer_handle: PeerHandle,
        data: &'data [u8],
    },
    Close {
        peer_handle: PeerHandle,
    },
    Dns {
        hostname: &'data str,
    },
}

#[cfg(feature = "defmt")]
impl defmt::Format for TxEvent<'_> {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            TxEvent::CanBeOpened { .. } => defmt::write!(fmt, "TxEvent::CanBeOpened"),
            TxEvent::Connect { .. } => defmt::write!(fmt, "TxEvent::Connect"),
            TxEvent::Send { .. } => defmt::write!(fmt, "TxEvent::Send"),
            TxEvent::Close { .. } => defmt::write!(fmt, "TxEvent::Close"),
            TxEvent::Dns { .. } => defmt::write!(fmt, "TxEvent::Dns"),
        }
    }
}
