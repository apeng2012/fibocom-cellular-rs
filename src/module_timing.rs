#![allow(clippy::if_same_then_else)]

use embassy_time::Duration;

/// Low time of `PWR_ON` pin to trigger module switch on from power off mode
pub fn pwr_on_time() -> Duration {
    Duration::from_millis(100)
}

/// Low time of `PWR_ON` pin to trigger module graceful switch off
pub fn pwr_off_time() -> Duration {
    Duration::from_secs(3)
}

/// Low time of `RESET_N` pin to trigger module reset (reboot)
pub fn reset_time() -> Duration {
    Duration::from_millis(200)
}

/// Time to wait for module to boot
pub fn boot_time() -> Duration {
    Duration::from_secs(10)
}
