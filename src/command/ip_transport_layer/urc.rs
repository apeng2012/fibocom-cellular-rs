//! Unsolicited responses for Internet protocol transport layer Commands
use atat::{atat_derive::AtatResp, heapless_bytes::Bytes};
use ublox_sockets::{PeerHandle, SocketHandle};

use super::types::OpenState;

/// +UUSORD/+UUSORF
#[derive(Debug, Clone, AtatResp)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SocketDataAvailable {
    #[at_arg(position = 0)]
    pub socket: SocketHandle,
    #[at_arg(position = 1)]
    pub length: usize,
}

/// +MIPOPEN
#[derive(Debug, Clone, AtatResp)]
pub struct SocketOpened {
    #[at_arg(position = 0)]
    pub id: PeerHandle,
    #[at_arg(position = 1)]
    pub state: OpenState,
    #[at_arg(position = 2)]
    pub listen_ip: Option<Bytes<12>>,
    #[at_arg(position = 3)]
    pub listen_port: Option<u16>,
}

/// +MIPCLOSE
#[derive(Debug, Clone, AtatResp)]
pub struct SocketClosed {
    #[at_arg(position = 0)]
    pub id: PeerHandle,
    #[at_arg(position = 1)]
    pub num_or_type: Option<u16>,
    #[at_arg(position = 2)]
    pub close_type: Option<u16>,
}
