pub mod responses;
pub mod types;

use atat::atat_derive::AtatCmd;
use responses::ResolveNameIpResponse;

/// Resolve name / IP number through DNS +MIPDNS
#[derive(Clone, AtatCmd)]
#[at_cmd("+MIPDNS", ResolveNameIpResponse, attempts = 1, timeout_ms = 20000)]
pub struct ResolveNameIp<'a> {
    #[at_arg(position = 1, len = 128)]
    pub ip_domain_string: &'a str,
}
