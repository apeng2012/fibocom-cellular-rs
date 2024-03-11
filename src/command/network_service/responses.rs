//! Responses for Network service Commands
use super::types::{
    NetworkRegistrationStat, NetworkRegistrationUrcConfig, OperatorNameFormat,
    OperatorSelectionMode, RadioAccessTechnologySelected, RatAct,
};
use atat::atat_derive::AtatResp;
use heapless::String;

/// 9.1.1 Extended signal quality +CSQ
#[derive(Debug, Clone, AtatResp)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SignalQuality {
    #[at_arg(position = 0)]
    pub rssi: u8,
    #[at_arg(position = 1)]
    pub ber: u8,
}

/// 7.5 Operator selection +COPS
#[derive(Clone, AtatResp)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct OperatorSelection {
    #[at_arg(position = 0)]
    pub mode: OperatorSelectionMode,
    #[at_arg(position = 1)]
    pub oper: Option<OperatorNameFormat>,
    #[at_arg(position = 2)]
    pub act: Option<RatAct>,
}

/// 7.8 Radio Access Technology (RAT) selection +URAT
#[derive(Clone, AtatResp)]
pub struct RadioAccessTechnology {
    #[at_arg(position = 0)]
    pub act: RadioAccessTechnologySelected,
}

/// 7.14 Network registration status +CREG
#[derive(Clone, AtatResp)]
pub struct NetworkRegistrationStatus {
    #[at_arg(position = 0)]
    pub n: NetworkRegistrationUrcConfig,
    #[at_arg(position = 1)]
    pub stat: NetworkRegistrationStat,
    #[at_arg(position = 2)]
    pub lac: Option<String<4>>,
    #[at_arg(position = 3)]
    pub ci: Option<String<8>>,
    #[at_arg(position = 4)]
    pub act_status: Option<u8>,
}
