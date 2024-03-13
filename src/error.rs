#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GenericError {
    Timeout,
    Clock,
    Unsupported,
}

#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum Error {
    // General device errors
    Overflow,
    BaudDetection,
    Busy,
    Uninitialized,
    StateTimeout,
    PoweredDown,
    AttachTimeout,
    ContextActivationTimeout,
    InvalidStateTransition,

    // Network errors
    Network,

    // Service specific errors
    // DataService(DataServiceError),

    // Generic shared errors, e.g. from `core::`
    Generic(GenericError),

    Atat(atat::Error),

    _Unknown,

    IoPin,

    SubscriberOverflow(embassy_sync::pubsub::Error),
}

#[cfg(feature = "defmt")]
impl defmt::Format for Error {
    fn format(&self, f: defmt::Formatter<'_>) {
        match self {
            Self::Overflow => defmt::write!(f, "Overflow"),
            Self::BaudDetection => defmt::write!(f, "BaudDetection"),
            Self::Busy => defmt::write!(f, "Busy"),
            Self::Uninitialized => defmt::write!(f, "Uninitialized"),
            Self::StateTimeout => defmt::write!(f, "StateTimeout"),
            Self::PoweredDown => defmt::write!(f, "PoweredDown"),
            Self::AttachTimeout => defmt::write!(f, "AttachTimeout"),
            Self::ContextActivationTimeout => defmt::write!(f, "ContextActivationTimeout"),
            Self::InvalidStateTransition => defmt::write!(f, "InvalidStateTransition"),
            Self::Network => defmt::write!(f, "Network"),
            // Self::DataService(e) => defmt::write!(f, "DataService({:?})", e),
            Self::Generic(e) => defmt::write!(f, "Generic({:?})", e),
            Self::Atat(e) => defmt::write!(f, "Atat({:?})", e),
            Self::_Unknown => defmt::write!(f, "_Unknown"),
            _ => defmt::write!(f, "non_exhaustive"),
        }
    }
}

impl From<atat::Error> for Error {
    fn from(e: atat::Error) -> Self {
        Self::Atat(e)
    }
}
