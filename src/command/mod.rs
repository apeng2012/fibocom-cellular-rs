//! AT Commands for u-blox cellular module family\
//! Following the [u-blox cellular modules AT commands manual](https://content.u-blox.com/sites/default/files/u-blox-CEL_ATCommands_UBX-13002752.pdf)

pub mod call_control;
pub mod control;
pub mod device_data_security;
pub mod device_lock;
pub mod dns;
pub mod file_system;
pub mod general;
#[cfg(feature = "internal-network-stack")]
pub mod ip_transport_layer;
pub mod ipc;
pub mod mobile_control;
pub mod network_service;
pub mod psn;
pub mod system_features;

use atat::{
    atat_derive::{AtatCmd, AtatResp, AtatUrc},
    digest::parser::urc_helper,
    nom::branch,
};

#[derive(Clone, AtatResp)]
pub struct NoResponse;

#[derive(Clone, AtatCmd)]
#[at_cmd("", NoResponse)]
pub struct AT;

#[derive(Debug, Clone)]
pub enum Urc {
    #[cfg(feature = "internal-network-stack")]
    SocketDataSentOver(ip_transport_layer::urc::SocketDataSentOver),
    #[cfg(feature = "internal-network-stack")]
    SocketClosed(ip_transport_layer::urc::SocketClosed),
    #[cfg(feature = "internal-network-stack")]
    SocketOpened(ip_transport_layer::urc::SocketOpened),
    #[cfg(feature = "internal-network-stack")]
    SocketDataIntoStack(ip_transport_layer::urc::SocketDataIntoStack),
    #[cfg(feature = "internal-network-stack")]
    DataConnectionActivated(psn::urc::DataConnectionActivated),
    #[cfg(feature = "internal-network-stack")]
    BrokenLink(ip_transport_layer::urc::BrokenLink),

    #[cfg(feature = "internal-network-stack")]
    SocketReadData(ip_transport_layer::urc::SocketReadData),
    #[cfg(feature = "internal-network-stack")]
    CanSocketOpen(ip_transport_layer::urc::CanSocketOpen),
}

#[derive(Debug, Clone, AtatUrc)]
pub enum UrcInner {
    #[cfg(feature = "internal-network-stack")]
    #[at_urc("+MIPSEND")]
    SocketDataSentOver(ip_transport_layer::urc::SocketDataSentOver),
    #[cfg(feature = "internal-network-stack")]
    #[at_urc("+MIPCLOSE")]
    SocketClosed(ip_transport_layer::urc::SocketClosed),
    #[cfg(feature = "internal-network-stack")]
    #[at_urc("+MIPOPEN")]
    SocketOpened(ip_transport_layer::urc::SocketOpened),
    #[cfg(feature = "internal-network-stack")]
    #[at_urc("+MIPPUSH")]
    SocketDataIntoStack(ip_transport_layer::urc::SocketDataIntoStack),
    #[cfg(feature = "internal-network-stack")]
    #[at_urc("+MIPCALL")]
    DataConnectionActivated(psn::urc::DataConnectionActivated),
    #[cfg(feature = "internal-network-stack")]
    #[at_urc("+MIPSTAT")]
    BrokenLink(ip_transport_layer::urc::BrokenLink),
}

impl From<UrcInner> for Urc {
    fn from(value: UrcInner) -> Self {
        match value {
            UrcInner::SocketDataSentOver(x) => Urc::SocketDataSentOver(x),
            UrcInner::SocketClosed(x) => Urc::SocketClosed(x),
            UrcInner::SocketOpened(x) => Urc::SocketOpened(x),
            UrcInner::SocketDataIntoStack(x) => Urc::SocketDataIntoStack(x),
            UrcInner::DataConnectionActivated(x) => Urc::DataConnectionActivated(x),
            UrcInner::BrokenLink(x) => Urc::BrokenLink(x),
        }
    }
}

impl atat::AtatUrc for Urc {
    type Response = Urc;

    fn parse(resp: &[u8]) -> Option<Self::Response> {
        if let Some(urc) = ip_transport_layer::complete::parse_read_data(resp) {
            Some(urc)
        } else if let Some(urc) = ip_transport_layer::complete::parse_can_socket_open(resp) {
            Some(urc)
        } else {
            UrcInner::parse(resp).map(|x| x.into())
        }
    }
}

impl atat::Parser for Urc {
    fn parse(buf: &[u8]) -> Result<(&[u8], usize), atat::digest::ParseError> {
        let (_, r) = branch::alt((
            ip_transport_layer::streaming::parse_read_data,
            ip_transport_layer::streaming::parse_can_socket_open,
            urc_helper("+MIPSEND"),
            urc_helper("+MIPCLOSE"),
            urc_helper("+MIPOPEN"),
            urc_helper("+MIPPUSH"),
            urc_helper("+MIPCALL"),
            urc_helper("+MIPSTAT"),
        ))(buf)?;
        Ok(r)
    }
}
