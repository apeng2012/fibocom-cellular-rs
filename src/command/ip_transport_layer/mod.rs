//! ### 25 - Internet protocol transport layer Commands
//!

pub mod responses;
pub mod types;
pub mod urc;

use super::NoResponse;
use atat::atat_derive::AtatCmd;
use embedded_nal::IpAddr;
use heapless::String;
use responses::{
    SocketControlResponse, SocketData, SocketErrorResponse, UDPSendToDataResponse, UDPSocketData,
    WriteSocketDataResponse,
};
use types::{SocketControlParam, SocketProtocol, SslTlsStatus};
use ublox_sockets::PeerHandle;
use ublox_sockets::SocketHandle;

/// 25.4 SSL/TLS mode configuration on TCP socket +USOSEC
///
/// Enables or disables the use of SSL/TLS connection on a TCP socket. The
/// configuration of the SSL/TLS properties is provided with an SSL/TLS profile
/// managed by USECMNG. The <`usecmng_profile_id`> parameter is listed in the
/// information text response to the read command only if the SSL/TLS is enabled
/// on the interested socket.
///
/// **Notes:**
/// - This operation is only available for TCP sockets
/// - The enable or disable operation can be performed only after the socket has
///   been created with +USOCR AT command.
/// - The SSL/TLS is supported only with +USOCO command (socket connect
///   command). The SSL/TLS is not supported with +USOLI command (socket set
///   listen command is not supported and the +USOSEC settings will be ignored).
#[derive(Clone, AtatCmd)]
#[at_cmd("+USOSEC", NoResponse)]
pub struct SetSocketSslState {
    // len 1 as ublox devices only support 7 sockets but needs to be changed if this changes!
    // FIXME: having all the lines use a constant something like  #[at_arg(position = 0, len = MAX_SOCKETS)]
    #[at_arg(position = 0, len = 1)]
    pub socket: SocketHandle,
    #[at_arg(position = 1)]
    pub ssl_tls_status: SslTlsStatus,
}

/// Close Socket +MIPCLOSE
#[derive(Clone, AtatCmd)]
#[at_cmd("+MIPCLOSE", NoResponse, attempts = 3, timeout_ms = 20000)]
pub struct CloseSocket {
    #[at_arg(position = 0, len = 1)]
    pub socket: PeerHandle,
}

/// 25.8 Get Socket Error +USOER
///
/// Retrieves the last error occurred in the last socket operation, stored in
/// the BSD standard variable error.
#[derive(Clone, AtatCmd)]
#[at_cmd("+USOER", SocketErrorResponse)]
pub struct GetSocketError;

/// Open a Socket +MIPOPEN
pub struct ConnectSocket {
    pub id: PeerHandle,
    pub port: Option<u16>,
    pub remote_addr: String<100>,
    pub remote_port: u16,
    pub protocol: SocketProtocol,
}

impl atat::AtatCmd for ConnectSocket {
    type Response = NoResponse;

    const MAX_LEN: usize = 128;
    const MAX_TIMEOUT_MS: u32 = 20_000u32;

    fn write(&self, mut buf: &mut [u8]) -> usize {
        assert!(buf.len() >= Self::MAX_LEN);
        let buf_len = buf.len();
        let id = self.id.0;
        use embedded_io::Write;
        if let Some(port) = self.port {
            write!(
                buf,
                "AT+MIPOPEN={},{},\"{}\",{},{}\r\n",
                id, port, self.remote_addr, self.remote_port, self.protocol as u8
            )
            .ok();
        } else {
            write!(
                buf,
                "AT+MIPOPEN={},,\"{}\",{},{}\r\n",
                id, self.remote_addr, self.remote_port, self.protocol as u8
            )
            .ok();
        }
        buf_len - buf.len()
    }

    fn parse(
        &self,
        _resp: Result<&[u8], atat::InternalError>,
    ) -> Result<Self::Response, atat::Error> {
        Ok(NoResponse)
    }
}

/// 25.10 Write socket data +USOWR
///
/// Writes the specified amount of data to the specified socket, like the BSD
/// write routine, and returns the number of bytes of data actually written. The
/// command applies to UDP sockets too, after a +USOCO command. There are three
/// kinds of syntax:
/// - Base syntax normal: writing simple strings to the socket, some characters
///   are forbidden
#[derive(Clone, AtatCmd)]
#[at_cmd("+USOWR", WriteSocketDataResponse)]
pub struct WriteSocketData<'a> {
    // len 1 as ublox devices only support 7 sockets but needs to be changed if this changes!
    #[at_arg(position = 0, len = 1)]
    pub socket: SocketHandle,
    #[at_arg(position = 1)]
    pub length: usize,
    #[at_arg(position = 2, len = 512)]
    pub data: &'a str,
}

/// 25.10 Write socket data +USOWR
///
/// Writes the specified amount of data to the specified socket, like the BSD
/// write routine, and returns the number of bytes of data actually written. The
/// command applies to UDP sockets too, after a +USOCO command. There are three
/// kinds of syntax:
/// - Base syntax HEX: writing hexadecimal strings to the socket, the string
///   will be converted in binary data and sent to the socket; see the
///   AT+UDCONF=1 command description to enable it
#[derive(Clone, AtatCmd)]
#[at_cmd("+USOWR", WriteSocketDataResponse)]
pub struct WriteSocketDataHex<'a> {
    // len 1 as ublox devices only support 7 sockets but needs to be changed if this changes!
    #[at_arg(position = 0, len = 1)]
    pub socket: SocketHandle,
    #[at_arg(position = 1)]
    pub length: usize,
    #[at_arg(position = 2, len = 512)]
    pub data: &'a [u8],
}

/// 25.10 Write socket data +USOWR
///
/// Writes the specified amount of data to the specified socket, like the BSD
/// write routine, and returns the number of bytes of data actually written. The
/// command applies to UDP sockets too, after a +USOCO command. There are three
/// kinds of syntax:
/// - Binary extended syntax: mandatory for writing any character in the ASCII
///   range [0x00, 0xFF]
#[derive(Clone, AtatCmd)]
#[at_cmd("+USOWR", NoResponse)]
pub struct PrepareWriteSocketDataBinary {
    // len 1 as ublox devices only support 7 sockets but needs to be changed if this changes!
    #[at_arg(position = 0, len = 1)]
    pub socket: SocketHandle,
    #[at_arg(position = 1)]
    pub length: usize,
}

#[derive(Clone, AtatCmd)]
#[at_cmd(
    "",
    WriteSocketDataResponse,
    value_sep = false,
    cmd_prefix = "",
    termination = "",
    force_receive_state = true
)]
pub struct WriteSocketDataBinary<'a> {
    // FIXME:
    // #[at_arg(position = 0, len = EgressChunkSize::to_usize())]
    #[at_arg(position = 0, len = 1024)]
    pub data: &'a atat::serde_bytes::Bytes,
}

///25.11 `SendTo` command (UDP only) +USOST
///
/// Writes the specified amount of data to the remote address,
/// like the BSD sendto routine, and returns the number of bytes
/// of data actually written. It can be applied to UDP sockets
/// only. This command allows the reuse of the same socket to send
/// data to many different remote hosts.
#[derive(Clone, AtatCmd)]
#[at_cmd("+USOST", NoResponse)]
pub struct PrepareUDPSendToDataBinary {
    // len 1 as ublox devices only support 7 sockets but needs to be changed if this changes!
    #[at_arg(position = 0, len = 1)]
    pub socket: SocketHandle,
    #[at_arg(position = 1, len = 39)]
    pub remote_addr: IpAddr,
    #[at_arg(position = 2)]
    pub remote_port: u16,
    #[at_arg(position = 3)]
    pub length: usize,
}

#[derive(Clone, AtatCmd)]
#[at_cmd(
    "",
    UDPSendToDataResponse,
    value_sep = false,
    cmd_prefix = "",
    termination = "",
    force_receive_state = true
)]
pub struct UDPSendToDataBinary<'a> {
    #[at_arg(position = 0, len = 512)]
    pub data: &'a atat::serde_bytes::Bytes,
}

/// 25.12 Read Socket Data +USORD
///
/// Reads the specified amount of data from the specified socket, like the BSD
/// read routine. This command can be used to know the total amount of unread
/// data.
///
/// For the TCP socket type the URC +UUSORD: <socket>,<length> notifies the data
/// bytes available for reading, either when buffer is empty and new data
/// arrives or after a partial read by the user.
///
/// For the UDP socket type the URC +UUSORD: <socket>,<length> notifies that a
/// UDP packet has been received, either when buffer is empty or after a UDP
/// packet has been read and one or more packets are stored in the buffer.
///
/// In case of a partial read of a UDP packet +UUSORD: <socket>,<length> will
/// show the remaining number of data bytes of the packet the user is reading.
#[derive(Clone, AtatCmd)]
#[at_cmd("+USORD", SocketData)]
pub struct ReadSocketData {
    // len 1 as ublox devices only support 7 sockets but needs to be changed if this changes!
    #[at_arg(position = 0, len = 1)]
    pub socket: SocketHandle,
    #[at_arg(position = 1)]
    pub length: usize,
}

/// 25.13 Receive From command (UDP only) +USORF
///
/// Reads the specified amount of data from the specified UDP socket, like the
/// BSD recvfrom routine. The URC +UUSORF: <socket>,<length> (or also +UUSORD:
/// <socket>,<length>) notifies that new data is available for reading, either
/// when new data arrives or after a partial read by the user for the socket.
/// This command can also return the total amount of unread data.
#[derive(Clone, AtatCmd)]
#[at_cmd("+USORF", UDPSocketData)]
pub struct ReadUDPSocketData {
    // len 1 as ublox devices only support 7 sockets but needs to be changed if this changes!
    #[at_arg(position = 0, len = 1)]
    pub socket: SocketHandle,
    #[at_arg(position = 1)]
    pub length: usize,
}

/// 25.16 HEX mode configuration +UDCONF=1
///
/// Enables/disables the HEX mode for +USOWR, +USOST, +USORD and +USORF AT
/// commands.
#[derive(Clone, AtatCmd)]
#[at_cmd("+GTSET=\"IPRFMT\",", NoResponse, value_sep = false)]
pub struct SetHexMode {
    /// 0: Received data with “+MIPRTCP:” and the data is encoded.
    /// 1: Received data only and the data are without encoded. In received
    ///    character string, Module doesn’t accede to any <CR><LF> symbol.
    /// 2: Received data with “+MIPRTCP:” and the data is without encoded. In
    ///    received character string, Module will accede to <CR><LF> before
    ///    “+MIPRTCP:”.
    /// 5: Data read mode
    /// The default value is 0.
    #[at_arg(position = 0)]
    pub hex_mode: u8,
}

/// 25.25 Socket control +USOCTL
///
/// Allows interaction with the low level socket layer.
#[derive(Clone, AtatCmd)]
#[at_cmd("+USOCTL", SocketControlResponse)]
pub struct SocketControl {
    // len 1 as ublox devices only support 7 sockets but needs to be changed if this changes!
    #[at_arg(position = 0, len = 1)]
    pub socket: SocketHandle,
    #[at_arg(position = 1)]
    pub param_id: SocketControlParam,
}
