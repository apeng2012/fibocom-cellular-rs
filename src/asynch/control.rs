use core::future::poll_fn;
use core::task::Poll;

use atat::asynch::AtatClient;

use crate::error::Error;

use super::state::{LinkState, OperationState};
use super::{state, AtHandle};

pub struct Control<'a, AT: AtatClient> {
    state_ch: state::StateRunner<'a>,
    at: AtHandle<'a, AT>,
}

impl<'a, AT: AtatClient> Control<'a, AT> {
    pub(crate) fn new(state_ch: state::StateRunner<'a>, at: AtHandle<'a, AT>) -> Self {
        Self { state_ch, at }
    }

    pub fn link_state(&mut self) -> Option<LinkState> {
        self.state_ch.link_state()
    }

    pub fn power_state(&mut self) -> OperationState {
        self.state_ch.power_state()
    }

    pub fn desired_state(&mut self) -> OperationState {
        self.state_ch.desired_state()
    }

    pub async fn set_desired_state(&mut self, ps: OperationState) {
        self.state_ch.set_desired_state(ps).await;
    }

    pub async fn set_desired_state_and_wait_for_completion(&mut self, ps: OperationState) {
        if self.state_ch.power_state() == ps {
            self.state_ch.set_power_state(OperationState::PowerDown);
        }
        self.state_ch.set_desired_state(ps).await;
        poll_fn(|cx| {
            if self.state_ch.power_state_poll_fn(cx) == ps {
                Poll::Ready(())
            } else {
                Poll::<()>::Pending
            }
        })
        .await;
    }

    pub async fn get_signal_quality(
        &mut self,
    ) -> Result<crate::command::network_service::responses::SignalQuality, Error> {
        self.at
            .send(&crate::command::network_service::GetSignalQuality)
            .await
            .map_err(|e| Error::Atat(e))
    }

    pub async fn get_operator(
        &mut self,
    ) -> Result<crate::command::network_service::responses::OperatorSelection, Error> {
        self.at
            .send(&crate::command::network_service::GetOperatorSelection)
            .await
            .map_err(|e| Error::Atat(e))
    }

    /// Send an AT command to the modem
    /// This is usefull if you have special configuration but might break the drivers functionality if your settings interfere with the drivers settings
    pub async fn send<Cmd: atat::AtatCmd>(
        &mut self,
        cmd: &Cmd,
    ) -> Result<Cmd::Response, atat::Error> {
        #[cfg(feature = "low-mcu")]
        {
            use embassy_time::Duration;
            use embassy_time::Timer;
            Timer::after(Duration::from_millis(100)).await;
        }
        let ret = self.at.send::<Cmd>(cmd).await;
        #[cfg(feature = "low-mcu")]
        {
            use embassy_time::Duration;
            use embassy_time::Timer;
            Timer::after(Duration::from_millis(100)).await;
        }
        ret
    }
}
