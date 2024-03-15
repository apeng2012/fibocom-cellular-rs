//! Responses for DNS Commands
use atat::{atat_derive::AtatResp, heapless_bytes::Bytes};
use heapless::String;

/// 24.1 Resolve name / IP number through DNS +UDNSRN
#[derive(Clone, PartialEq, Eq, AtatResp)]
pub struct ResolveNameIpResponse {
    #[at_arg(position = 0)]
    pub ip_domain_string: String<128>,
    #[at_arg(position = 1)]
    pub ip_addr: Bytes<16>,
}
