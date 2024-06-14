use crate::{command::Urc, config::CellularConfig};

use super::state;
use crate::asynch::state::{LinkState, OperationState};
use crate::command;
use crate::command::control::types::FlowControl;
use crate::command::control::SetFlowControl;
use crate::command::device_lock::responses::PinStatus;
use crate::command::device_lock::types::PinStatusCode;
use crate::command::device_lock::GetPinStatus;
use crate::command::general::{GetCCID, GetFirmwareVersion, GetModelId};
use crate::command::mobile_control::types::{Functionality, TerminationErrorMode};
use crate::command::mobile_control::{SetModuleFunctionality, SetReportMobileTerminationError};
use crate::command::psn::responses::GPRSNetworkRegistrationStatus;
use crate::command::psn::types::GPRSNetworkRegistrationStat;
use crate::command::psn::GetGPRSNetworkRegistrationStatus;
use crate::command::AT;
use crate::config::Apn;
use crate::error::Error;
use crate::module_timing::{boot_time, reset_time};
use atat::{asynch::AtatClient, UrcSubscription};
use embassy_futures::select::select;
use embassy_time::{with_timeout, Duration, Timer};
use embedded_hal::digital::{InputPin, OutputPin};
use heapless::String;

use crate::command::psn::types::{ContextId, ProfileId};
use embassy_futures::select::Either;

use super::AtHandle;

/// Background runner for the Ublox Module.
///
/// You must call `.run()` in a background task for the Ublox Module to operate.
pub struct Runner<'d, AT: AtatClient, C: CellularConfig<'d>, const URC_CAPACITY: usize> {
    ch: state::Runner<'d>,
    at: AtHandle<'d, AT>,
    config: C,
    urc_subscription: UrcSubscription<'d, Urc, URC_CAPACITY, 2>,
}

impl<'d, AT: AtatClient, C: CellularConfig<'d>, const URC_CAPACITY: usize>
    Runner<'d, AT, C, URC_CAPACITY>
{
    pub(crate) fn new(
        ch: state::Runner<'d>,
        at: AtHandle<'d, AT>,
        config: C,
        urc_subscription: UrcSubscription<'d, Urc, URC_CAPACITY, 2>,
    ) -> Self {
        Self {
            ch,
            at,
            config,
            urc_subscription,
        }
    }

    // TODO: crate visibility only makes sense if reset and co are also crate visibility
    // pub(crate) async fn init(&mut self) -> Result<(), Error> {
    pub async fn init(&mut self) -> Result<(), Error> {
        // Initilize a new ublox device to a known state (set RS232 settings)
        debug!("Initializing module");
        // Hard reset module
        if Ok(false) == self.has_power().await {
            self.power_up().await?;
        };
        self.reset().await?;
        // self.is_alive().await?;

        Ok(())
    }

    pub async fn is_alive(&mut self) -> Result<bool, Error> {
        if !self.has_power().await? {
            return Err(Error::PoweredDown);
        }

        match self.at.send(&AT).await {
            Ok(_) => Ok(true),
            Err(err) => Err(Error::Atat(err)),
        }
    }

    pub async fn has_power(&mut self) -> Result<bool, Error> {
        if let Some(pin) = self.config.vint_pin() {
            if pin.is_high().map_err(|_| Error::IoPin)? {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            info!("No VInt pin configured");
            Ok(true)
        }
    }

    pub async fn power_up(&mut self) -> Result<(), Error> {
        if let Some(pin) = self.config.reset_pin() {
            pin.set_high().map_err(|_| Error::IoPin)?;
        }
        if let Some(pin) = self.config.power_pin() {
            pin.set_high().map_err(|_| Error::IoPin)?;
            Timer::after(boot_time()).await;
            self.ch.set_power_state(OperationState::PowerUp);
            warn!("Powered up");
            Ok(())
        } else {
            warn!("No power pin configured");
            Ok(())
        }
    }

    pub async fn power_down(&mut self) -> Result<(), Error> {
        if let Some(pin) = self.config.power_pin() {
            pin.set_low().map_err(|_| Error::IoPin)?;
            Timer::after(crate::module_timing::pwr_off_time()).await;
            pin.set_high().map_err(|_| Error::IoPin)?;
            self.ch.set_power_state(OperationState::PowerDown);
            debug!("Powered down");
            Ok(())
        } else {
            warn!("No power pin configured");
            Ok(())
        }
    }

    pub async fn init_at(&mut self) -> Result<(), Error> {
        if !self.is_alive().await? {
            return Err(Error::PoweredDown);
        }

        // Extended errors on
        self.at
            .send(&SetReportMobileTerminationError {
                n: TerminationErrorMode::Enabled,
            })
            .await?;

        let _model_id = self.at.send(&GetModelId).await?;

        self.at.send(&GetFirmwareVersion).await?;

        self.select_sim_card().await?;

        let _ccid = self.at.send(&GetCCID).await?;

        #[cfg(feature = "internal-network-stack")]
        if C::HEX_MODE {
            self.at
                .send(&crate::command::ip_transport_layer::SetHexMode { hex_mode: 2 })
                .await?;
        } else {
            unreachable!();
        }

        // Tell module whether we support flow control
        // FIXME: Use AT+IFC=2,2 instead of AT&K here
        if C::FLOW_CONTROL {
            self.at
                .send(&SetFlowControl {
                    value: FlowControl::RtsCts,
                })
                .await?;
        } else {
            self.at
                .send(&SetFlowControl {
                    value: FlowControl::Disabled,
                })
                .await?;
        }
        Ok(())
    }
    /// Initializes the network only valid after `init_at`.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the internal network operations fail.
    ///
    pub async fn init_network(&mut self) -> Result<(), Error> {
        self.at
            .send(&crate::command::network_service::GetSignalQuality)
            .await?;

        self.at
            .send(
                &crate::command::mobile_control::SetAutomaticTimezoneUpdate {
                    on_off: crate::command::mobile_control::types::AutomaticTimezone::EnabledLocal,
                },
            )
            .await?;

        self.at
            .send(&crate::command::mobile_control::SetModuleFunctionality {
                fun: Functionality::Full,
            })
            .await?;

        self.enable_registration_urcs().await?;

        // Set automatic operator selection, if not already set
        let crate::command::network_service::responses::OperatorSelection { mode, .. } = self
            .at
            .send(&crate::command::network_service::GetOperatorSelection)
            .await?;

        // Only run AT+COPS=0 if currently de-registered, to avoid PLMN reselection
        if !matches!(
            mode,
            crate::command::network_service::types::OperatorSelectionMode::Automatic
                | crate::command::network_service::types::OperatorSelectionMode::Manual
        ) {
            self.at
                .send(&crate::command::network_service::SetOperatorSelection {
                    mode: crate::command::network_service::types::OperatorSelectionMode::Automatic,
                    format: Some(C::OPERATOR_FORMAT as u8),
                })
                .await?;
        }

        Ok(())
    }

    pub(crate) async fn enable_registration_urcs(&mut self) -> Result<(), Error> {
        // if packet domain event reporting is not set it's not a stopper. We
        // might lack some events when we are dropped from the network.
        // TODO: Re-enable this when it works, and is useful!
        if self
            .at
            .send(&crate::command::psn::SetPacketSwitchedEventReporting {
                mode: crate::command::psn::types::PSEventReportingMode::CircularBufferUrcs,
                bfr: None,
            })
            .await
            .is_err()
        {
            warn!("Packet domain event reporting set failed");
        }

        // FIXME: Currently `atat` is unable to distinguish `xREG` family of
        // commands from URC's

        // CREG URC
        self.at.send(
            &crate::command::network_service::SetNetworkRegistrationStatus {
                n: crate::command::network_service::types::NetworkRegistrationUrcConfig::UrcDisabled,
            }).await?;

        // CGREG URC
        self.at
            .send(&crate::command::psn::SetGPRSNetworkRegistrationStatus {
                n: crate::command::psn::types::GPRSNetworkRegistrationUrcConfig::UrcDisabled,
            })
            .await?;

        // CEREG URC
        self.at
            .send(&crate::command::psn::SetEPSNetworkRegistrationStatus {
                n: crate::command::psn::types::EPSNetworkRegistrationUrcConfig::UrcDisabled,
            })
            .await?;

        Ok(())
    }

    pub async fn select_sim_card(&mut self) -> Result<(), Error> {
        for _ in 0..2 {
            match self.at.send(&GetPinStatus).await {
                Ok(PinStatus { code }) if code == PinStatusCode::Ready => {
                    debug!("SIM is ready");
                    return Ok(());
                }
                _ => {}
            }

            Timer::after(Duration::from_secs(1)).await;
        }

        // There was an error initializing the SIM
        // We've seen issues on uBlox-based devices, as a precation, we'll cycle
        // the modem here through minimal/full functional state.
        self.at
            .send(&SetModuleFunctionality {
                fun: Functionality::TrunOff,
            })
            .await?;
        self.at
            .send(&SetModuleFunctionality {
                fun: Functionality::Full,
            })
            .await?;

        Ok(())
    }

    /// Reset the module by driving it's `RESET_N` pin low for 50 ms
    ///
    /// **NOTE** This function will reset NVM settings!
    pub async fn reset(&mut self) -> Result<(), Error> {
        warn!("Hard resetting Ublox Cellular Module");
        if let Some(pin) = self.config.reset_pin() {
            pin.set_low().ok();
            Timer::after(reset_time()).await;
            pin.set_high().ok();
            Timer::after(boot_time()).await;
            // self.is_alive().await?;
        } else {
            warn!("No reset pin configured");
        }
        Ok(())
    }

    /// Perform at full factory reset of the module, clearing all NVM sectors in the process
    pub async fn factory_reset(&mut self) -> Result<(), Error> {
        self.at
            .send(&crate::command::system_features::SetFactoryConfiguration {
                fs_op: crate::command::system_features::types::FSFactoryRestoreType::AllFiles,
                nvm_op:
                    crate::command::system_features::types::NVMFactoryRestoreType::NVMFlashSectors,
            })
            .await?;

        info!("Successfully factory reset modem!");

        if self.soft_reset(true).await.is_err() {
            self.reset().await?;
        }

        Ok(())
    }

    /// Reset the module by sending AT CFUN command
    pub async fn soft_reset(&mut self, sim_reset: bool) -> Result<(), Error> {
        trace!(
            "Attempting to soft reset of the modem with sim reset: {}.",
            sim_reset
        );

        match self
            .at
            .send(&SetModuleFunctionality {
                fun: Functionality::SilentReset,
            })
            .await
        {
            Ok(_) => {
                info!("Successfully soft reset modem!");
                Ok(())
            }
            Err(err) => {
                error!("Failed to soft reset modem: {:?}", err);
                Err(Error::Atat(err))
            }
        }
    }

    // checks alive status continuiously until it is alive
    async fn check_is_alive_loop(&mut self) -> bool {
        loop {
            if let Ok(alive) = self.is_alive().await {
                return alive;
            }
            Timer::after(Duration::from_millis(100)).await;
        }
    }

    async fn is_network_attached_loop(&mut self) -> bool {
        loop {
            if let Ok(true) = self.is_network_attached().await {
                return true;
            }
            Timer::after(Duration::from_secs(1)).await;
        }
    }

    pub async fn run(&mut self) -> ! {
        match self.has_power().await.ok() {
            Some(false) => {
                self.ch.set_power_state(OperationState::PowerDown);
            }
            Some(true) => {
                self.ch.set_power_state(OperationState::PowerUp);
            }
            None => {
                self.ch.set_power_state(OperationState::PowerDown);
            }
        }
        loop {
            match select(
                self.ch.state_runner().wait_for_desired_state_change(),
                self.urc_subscription.next_message_pure(),
            )
            .await
            {
                Either::First(desired_state) => {
                    info!("Desired state: {:?}", desired_state);
                    if let Err(err) = desired_state {
                        error!("Error in desired_state retrival: {:?}", err);
                        continue;
                    }
                    let desired_state = desired_state.unwrap();
                    while let Err(e) = self.change_state_to_desired_state(desired_state).await {
                        error!(
                            "can not change_state_to_desired_state {:?}. power off and try again.",
                            e
                        );
                        self.power_down().await.ok();
                        self.ch.set_power_state(OperationState::PowerDown);
                    }
                }
                Either::Second(event) => {
                    self.handle_urc(event).await.ok();
                }
            }
        }
    }

    /// When desired state is the current power_state, verify the status starting from the PowerDown state
    async fn change_state_to_desired_state(
        &mut self,
        desired_state: OperationState,
    ) -> Result<(), Error> {
        let dir = desired_state as isize - self.ch.state_runner().power_state() as isize;
        if 0 >= dir {
            debug!("Power steps was negative, power down: {}", dir);
            if 0 != dir || OperationState::PowerDown == desired_state {
                self.power_down().await.ok();
            }
            self.ch.set_power_state(OperationState::PowerDown);
        }
        let start_state = self.ch.state_runner().power_state() as isize;
        let steps = desired_state as isize - start_state;
        for step in 0..=steps {
            debug!(
                "State transition {} steps: {} -> {}, {}",
                steps,
                start_state,
                start_state + step,
                step
            );
            let next_state = start_state + step;
            match OperationState::try_from(next_state) {
                Ok(OperationState::PowerDown) => {}
                Ok(OperationState::PowerUp) => match self.power_up().await {
                    Ok(_) => {
                        self.ch.set_power_state(OperationState::PowerUp);
                    }
                    Err(err) => {
                        error!("Error in power_up: {:?}", err);
                        return Err(err);
                    }
                },
                Ok(OperationState::Alive) => {
                    match with_timeout(boot_time() * 2, self.check_is_alive_loop()).await {
                        Ok(true) => {
                            debug!("Will set Alive");
                            self.ch.set_power_state(OperationState::Alive);
                            debug!("Set Alive");
                        }
                        Ok(false) => {
                            error!("Error in is_alive: {:?}", Error::PoweredDown);
                            return Err(Error::PoweredDown);
                        }
                        Err(err) => {
                            error!("Error in is_alive: {:?}", err);
                            return Err(Error::StateTimeout);
                        }
                    }
                }
                Ok(OperationState::Initialized) => {
                    #[cfg(not(feature = "ppp"))]
                    match self.init_at().await {
                        Ok(_) => {
                            self.ch.set_power_state(OperationState::Initialized);
                        }
                        Err(err) => {
                            error!("Error in init_at: {:?}", err);
                            return Err(err);
                        }
                    }

                    #[cfg(feature = "ppp")]
                    {
                        self.ch.set_power_state(OperationState::Initialized);
                    }
                }
                Ok(OperationState::Connected) => match self.init_network().await {
                    Ok(_) => {
                        match with_timeout(Duration::from_secs(50), self.is_network_attached_loop())
                            .await
                        {
                            Ok(_) => {
                                debug!("Will set Connected");
                                self.ch.set_power_state(OperationState::Connected);
                                debug!("Set Connected");
                            }
                            Err(err) => {
                                error!("Timeout waiting for network attach: {:?}", err);
                                return Err(Error::StateTimeout);
                            }
                        }
                    }
                    Err(err) => {
                        error!("Error in init_network: {:?}", err);
                        return Err(err);
                    }
                },
                #[cfg(not(feature = "ppp"))]
                Ok(OperationState::DataEstablished) => {
                    match self.connect(C::APN, C::PROFILE_ID, C::CONTEXT_ID).await {
                        Ok(_) => {
                            self.ch.set_power_state(OperationState::DataEstablished);
                        }
                        Err(err) => {
                            error!("Error in connect: {:?}", err);
                            return Err(err);
                        }
                    }
                }
                Err(_) => {
                    error!("State transition next_state not valid: start_state={}, next_state={}, steps={} ", start_state, next_state, steps);
                    return Err(Error::InvalidStateTransition);
                }
            }
        }
        Ok(())
    }

    async fn handle_urc(&mut self, event: Urc) -> Result<(), Error> {
        match event {
            // Handle network URCs
            #[cfg(feature = "internal-network-stack")]
            Urc::SocketReadData(_) => warn!("Socket read data"),
            #[cfg(feature = "internal-network-stack")]
            Urc::SocketDataSentOver(_) => warn!("Socket data sent over"),
            #[cfg(feature = "internal-network-stack")]
            Urc::DataConnectionActivated(dca) => {
                warn!("Data connection activated");
                self.ch.set_link_state(dca.sc.into());
            }
            #[cfg(feature = "internal-network-stack")]
            Urc::SocketClosed(_) => warn!("Socket closed"),
            #[cfg(feature = "internal-network-stack")]
            Urc::SocketOpened(_) => warn!("Socket opened"),
            #[cfg(feature = "internal-network-stack")]
            Urc::CanSocketOpen(_) => warn!("Socket can open"),
            #[cfg(feature = "internal-network-stack")]
            Urc::SocketDataIntoStack(_) => warn!("SocketDataIntoStack"),
            #[cfg(feature = "internal-network-stack")]
            Urc::BrokenLink(_) => warn!("Broken protocol stack"),
        };
        Ok(())
    }

    #[allow(unused_variables)]
    #[cfg(not(feature = "ppp"))]
    async fn connect(
        &mut self,
        apn_info: crate::config::Apn<'_>,
        profile_id: ProfileId,
        context_id: ContextId,
    ) -> Result<(), Error> {
        // This step _shouldn't_ be necessary.  However, for reasons I don't
        // understand, SARA-R4 can be registered but not attached (i.e. AT+CGATT
        // returns 0) on both RATs (unh?).  Phil Ware, who knows about these
        // things, always goes through (a) register, (b) wait for AT+CGATT to
        // return 1 and then (c) check that a context is active with AT+CGACT or
        // using AT+UPSD (even for EUTRAN). Since this sequence works for both
        // RANs, it is best to be consistent.
        let mut attached = false;
        for _ in 0..10 {
            attached = self.is_network_attached().await?;
            if attached {
                break;
            }
        }
        if !attached {
            return Err(Error::AttachTimeout);
        }

        // Activate the context
        self.activate_context(apn_info).await?;

        Ok(())
    }

    // Make sure we are attached to the cellular network.
    async fn is_network_attached(&mut self) -> Result<bool, Error> {
        let GPRSNetworkRegistrationStatus { stat, .. } = self
            .at
            .send(&GetGPRSNetworkRegistrationStatus)
            .await
            .map_err(Error::from)?;

        if stat == GPRSNetworkRegistrationStat::Registered
            || stat == GPRSNetworkRegistrationStat::RegisteredRoaming
        {
            return Ok(true);
        }
        return Ok(false);
    }

    async fn is_connected(&mut self) -> Result<LinkState, Error> {
        self.ch.set_link_state(None);

        for _ in 0..20 {
            self.at.send(&command::psn::GetStatusIp).await.ok();

            if let Ok(event) = with_timeout(
                Duration::from_millis(200),
                self.urc_subscription.next_message_pure(),
            )
            .await
            {
                self.handle_urc(event).await?;
            }

            if let Some(ls) = self.ch.state_runner().link_state() {
                return Ok(ls);
            }
        }

        Err(Error::_Unknown)
    }

    async fn activate_context(&mut self, apn_info: Apn<'_>) -> Result<(), Error> {
        if self.is_connected().await? == LinkState::Down {
            if let Apn::Given { name, .. } = apn_info {
                for _ in 0..3 {
                    while let Some(event) = self.urc_subscription.try_next_message_pure() {
                        self.handle_urc(event).await?;
                    }

                    if let Some(ls) = self.ch.state_runner().link_state() {
                        if ls == LinkState::Up {
                            return Ok(());
                        }
                    }

                    self.at
                        .send(&crate::command::psn::SetPacketSwitchedConfig {
                            apn: String::<99>::try_from(name).unwrap(),
                        })
                        .await
                        .map_err(Error::from)?;

                    if let Ok(event) = with_timeout(
                        Duration::from_secs(30),
                        self.urc_subscription.next_message_pure(),
                    )
                    .await
                    {
                        self.handle_urc(event).await?;
                    }
                }
            }
            return Err(Error::_Unknown);
        }
        Ok(())
    }
}
