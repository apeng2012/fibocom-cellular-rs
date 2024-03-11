pub mod control;
mod resources;
pub mod runner;
pub mod state;

#[cfg(feature = "internal-network-stack")]
mod internal_stack;
#[cfg(feature = "internal-network-stack")]
pub use internal_stack::{new_internal, InternalRunner, Resources};

use atat::asynch::AtatClient;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};

pub struct AtHandle<'d, AT: AtatClient>(&'d Mutex<NoopRawMutex, AT>);

impl<'d, AT: AtatClient> AtHandle<'d, AT> {
    async fn send<Cmd: atat::AtatCmd>(&mut self, cmd: &Cmd) -> Result<Cmd::Response, atat::Error> {
        #[cfg(feature = "low-mcu")]
        {
            use embassy_time::Duration;
            use embassy_time::Timer;
            Timer::after(Duration::from_millis(100)).await;
        }
        let ret = self.0.lock().await.send_retry::<Cmd>(cmd).await;
        #[cfg(feature = "low-mcu")]
        {
            use embassy_time::Duration;
            use embassy_time::Timer;
            Timer::after(Duration::from_millis(100)).await;
        }
        ret
    }
}
