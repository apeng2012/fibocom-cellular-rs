//! Argument and parameter types used by Mobile equipment control and status Commands and Responses
use atat::atat_derive::AtatEnum;

#[derive(Clone, PartialEq, Eq, AtatEnum)]
pub enum Functionality {
    /// 0: Turn off (With logging out network)
    TrunOff = 0,

    /// 1 Full functionality meaning start up MS(from offline mode)
    Full = 1,

    /// 4: Disables both transmit and receive RF circuits
    AirplaneMode = 4,

    /// 15: Hardware reset.(Need re-turn on the module)
    SilentReset = 15,
}

/// Automatic time zone update
#[derive(Clone, PartialEq, Eq, AtatEnum)]
pub enum AutomaticTimezone {
    /// 0: automatic time zone via NITZ disabled
    Disabled = 0,
    /// 1: automatic time zone update
    /// via NITZ enabled; if the network supports the service, update the local
    /// time to the module (not only time zone)
    EnabledLocal = 1,
    /// 2: automatic time zone update
    /// via NITZ enabled; if the network supports the service, update the GMT
    /// time to the module (not only time zone)
    EnabledGMT = 2,
}

#[derive(Clone, PartialEq, Eq, AtatEnum)]
pub enum TerminationErrorMode {
    /// 0: +CME ERROR: <err> result code disabled and ERROR used
    Disabled = 0,
    /// 1: +CME ERROR: <err> result code enabled and numeric <err> values used
    Enabled = 1,
    /// 2: +CME ERROR: <err> result code enabled and verbose <err> values used
    Verbose = 2,
}

#[derive(Clone, PartialEq, Eq, AtatEnum)]
pub enum PowerMode {
    ///MT is switched on with minimum functionality
    Minimum = 0,
    ///MT is switched on
    On = 1,
    ///MT is in "airplane mode"
    AirplaneMode = 4,
    ///MT is in "test mode"
    TestMode = 5,
    ///MT is in minimum functionality with SIM deactivated
    MinimumWithoutSim = 19,
}

#[derive(Clone, PartialEq, Eq, AtatEnum)]
pub enum STKMode {
    ///the SIM-toolkit interface in dedicated mode and fetching of proactive commands by SIM-APPL from the SIM-card are enabled
    DedicatedMode = 6,
    /// the SIM-toolkit interface is disabled; fetching of proactive commands by SIM-APPL from the SIM-card is enabled
    Disabled = 0,
    ///the SIM-toolkit interface in raw mode and fetching of proactive commands by SIM-APPL from the SIM-card are enabled
    RawMode = 9,
}

#[derive(Clone, PartialEq, Eq, AtatEnum)]
pub enum ReportMobileTerminationErrorStatus {
    ///+CME ERROR: <err> result code disabled and ERROR used
    DisabledERRORused = 0,
    ///+CME ERROR: <err> result code enabled and numeric <err> values used
    EnabledCodeUsed = 1,
    ///+CME ERROR: <err> result code enabled and verbose <err> values used
    EnabledVerbose = 2,
}
