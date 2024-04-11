//! Unsolicited responses for Internet protocol transport layer Commands
use atat::{atat_derive::AtatResp, heapless_bytes::Bytes};
use heapless::Vec;
use ublox_sockets::PeerHandle;

use super::types::{OpenState, SendStatus};

pub const DATA_PACKAGE_SIZE: usize = 2048;

/// +MIPRTCP
#[derive(Debug, Clone, AtatResp)]
pub struct SocketReadData {
    #[at_arg(position = 0)]
    pub id: PeerHandle,
    #[at_arg(position = 1)]
    pub length: usize,
    #[at_arg(position = 2)]
    pub data: Vec<u8, DATA_PACKAGE_SIZE>,
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

/// +MIPOPEN?
/// for each socket that can be opened
#[derive(Debug, Clone)]
pub struct CanSocketOpen {
    pub id_list: Vec<PeerHandle, 6>,
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

/// +MIPSEND
#[derive(Debug, Clone, AtatResp)]
pub struct SocketDataSentOver {
    #[at_arg(position = 0)]
    pub id: PeerHandle,
    #[at_arg(position = 1)]
    pub status: SendStatus,
    #[at_arg(position = 2)]
    pub free_size: u16,
}

/// +MIPPUSH
#[derive(Debug, Clone, AtatResp)]
pub struct SocketDataIntoStack {
    #[at_arg(position = 0)]
    pub id: PeerHandle,
    #[at_arg(position = 1)]
    pub status: SendStatus,
    #[at_arg(position = 2)]
    pub accumulated: Option<usize>,
}

/// +MIPSTAT
#[derive(Debug, Clone, AtatResp)]
pub struct BrokenLink {
    #[at_arg(position = 0)]
    pub id: PeerHandle,
    #[at_arg(position = 1)]
    pub n: u8,
    #[at_arg(position = 2)]
    pub acknowledged: Option<usize>,
}
