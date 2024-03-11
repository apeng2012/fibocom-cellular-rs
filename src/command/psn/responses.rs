//! Responses for Packet Switched Data Services Commands
use super::types::{
    ContextId, EPSNetworkRegistrationStat, EPSNetworkRegistrationUrcConfig,
    ExtendedPSNetworkRegistrationState, ExtendedPSNetworkRegistrationUrcConfig, GPRSAttachedState,
    GPRSNetworkRegistrationStat, GPRSNetworkRegistrationUrcConfig, PDPContextStatus,
    PacketSwitchedParam, ProfileId,
};
use crate::command::network_service::types::RatAct;
use atat::atat_derive::AtatResp;
use heapless::String;

// 18.7 Packet switched data configuration +UPSD Sets or reads all the
//  parameters in a specific packet switched data (PSD) profile. The command is
//  used to set up the PDP context parameters for an internal context, i.e. a
//  data connection using the internal IP stack and related AT commands for
//  sockets. To set all the parameters of the PSD profile a set command for each
//  parameter needs to be issued.
#[derive(AtatResp)]
pub struct PacketSwitchedConfig {
    #[at_arg(position = 0)]
    pub profile_id: ProfileId,
    #[at_arg(position = 1)]
    pub param: PacketSwitchedParam,
}

/// 18.14 GPRS attach or detach +CGATT Register (attach) the MT to, or
/// deregister (detach) the MT from the GPRS service. After this command the MT
/// remains in AT command mode. If the MT is already in the requested state
/// (attached or detached), the command is ignored and OK result code is
/// returned. If the requested state cannot be reached, an error result code is
/// returned. The command can be aborted if a character is sent to the DCE
/// during the command execution. Any active PDP context will be automatically
/// deactivated when the GPRS registration state changes to detached.
#[derive(AtatResp)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GPRSAttached {
    #[at_arg(position = 0)]
    pub state: GPRSAttachedState,
}

/// 18.16 PDP context activate or deactivate +CGACT
#[derive(Clone, AtatResp)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PDPContextState {
    #[at_arg(position = 0)]
    pub cid: ContextId,
    #[at_arg(position = 1)]
    pub status: PDPContextStatus,
}

/// 18.27 GPRS network registration status +CGREG
#[derive(Clone, AtatResp)]
pub struct GPRSNetworkRegistrationStatus {
    #[at_arg(position = 0)]
    pub n: GPRSNetworkRegistrationUrcConfig,
    #[at_arg(position = 1)]
    pub stat: GPRSNetworkRegistrationStat,
    #[at_arg(position = 2)]
    pub lac: Option<String<4>>,
    #[at_arg(position = 3)]
    pub ci: Option<String<8>>,
    #[at_arg(position = 4)]
    pub act: Option<RatAct>,
    #[at_arg(position = 5)]
    pub rac: Option<String<2>>,
}

/// 18.28 Extended network registration status +UREG
#[derive(Clone, AtatResp)]
pub struct ExtendedPSNetworkRegistrationStatus {
    #[at_arg(position = 0)]
    pub n: ExtendedPSNetworkRegistrationUrcConfig,
    #[at_arg(position = 1)]
    pub state: ExtendedPSNetworkRegistrationState,
}

/// 18.36 EPS network registration status +CEREG
#[derive(Clone, AtatResp)]
pub struct EPSNetworkRegistrationStatus {
    #[at_arg(position = 0)]
    pub n: EPSNetworkRegistrationUrcConfig,
    #[at_arg(position = 1)]
    pub stat: EPSNetworkRegistrationStat,
    #[at_arg(position = 2)]
    pub tac: Option<String<4>>,
    #[at_arg(position = 3)]
    pub ci: Option<String<8>>,
    #[at_arg(position = 4)]
    pub act: Option<RatAct>,
}
