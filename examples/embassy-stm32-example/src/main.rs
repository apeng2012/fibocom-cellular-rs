#![no_std]
#![no_main]
#![allow(stable_features)]
// #![feature(type_alias_impl_trait)]

use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, Output};
use embassy_stm32::peripherals::USART3;
use embassy_stm32::usart::{BufferedUart, BufferedUartRx, BufferedUartTx};
use embassy_stm32::{bind_interrupts, peripherals, Config};
use embassy_time::{Duration, Timer};
use static_cell::StaticCell;
use ublox_cellular::asynch::state::OperationState;
use ublox_cellular::asynch::InternalRunner;
use ublox_cellular::asynch::Resources;
use ublox_cellular::config::{Apn, CellularConfig, ReverseOutputPin};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USART3 => embassy_stm32::usart::BufferedInterruptHandler<peripherals::USART3>;
});

const CMD_BUF_SIZE: usize = 128;
const INGRESS_BUF_SIZE: usize = 1024;
const URC_CAPACITY: usize = 2;

struct MyCelullarConfig {
    reset_pin: Option<Output<'static>>,
    // reset_pin: Option<NoPin>,
    power_pin: Option<ReverseOutputPin<Output<'static>>>,
    // power_pin: Option<NoPin>,
    vint_pin: Option<Input<'static>>,
    // vint_pin: Option<NoPin>
}

impl<'a> CellularConfig<'a> for MyCelullarConfig {
    type ResetPin = Output<'static>;
    // type ResetPin = NoPin;
    type PowerPin = ReverseOutputPin<Output<'static>>;
    // type PowerPin = NoPin;
    type VintPin = Input<'static>;
    // type VintPin = NoPin;

    const FLOW_CONTROL: bool = false;
    const HEX_MODE: bool = true;
    const APN: Apn<'a> = Apn::Given {
        name: "hologram",
        username: None,
        password: None,
    };
    fn reset_pin(&mut self) -> Option<&mut Self::ResetPin> {
        info!("reset_pin");
        return self.reset_pin.as_mut();
    }
    fn power_pin(&mut self) -> Option<&mut Self::PowerPin> {
        info!("power_pin");
        return self.power_pin.as_mut();
    }
    fn vint_pin(&mut self) -> Option<&mut Self::VintPin> {
        info!("vint_pin = {}", self.vint_pin.as_mut()?.is_high());
        return self.vint_pin.as_mut();
    }
}

#[embassy_executor::main]
async fn main_task(spawner: Spawner) {
    let config = Config::default();
    let p = embassy_stm32::init(config);

    let mut uart_config = embassy_stm32::usart::Config::default();
    {
        uart_config.baudrate = 115200;
        uart_config.parity = embassy_stm32::usart::Parity::ParityNone;
        uart_config.stop_bits = embassy_stm32::usart::StopBits::STOP1;
        uart_config.data_bits = embassy_stm32::usart::DataBits::DataBits8;
    }

    static TX_BUF: StaticCell<[u8; 16]> = StaticCell::new();
    static RX_BUF: StaticCell<[u8; 16]> = StaticCell::new();

    let cell_uart = BufferedUart::new(
        p.USART3,
        Irqs,
        p.PB11,
        p.PB10,
        TX_BUF.init([0u8; 16]),
        RX_BUF.init([0u8; 16]),
        uart_config,
    )
    .unwrap();
    let (uart_tx, uart_rx) = cell_uart.split();

    static RESOURCES: StaticCell<
        Resources<BufferedUartTx<USART3>, CMD_BUF_SIZE, INGRESS_BUF_SIZE, URC_CAPACITY>,
    > = StaticCell::new();

    let (_net_device, mut control, runner) = ublox_cellular::asynch::new_internal(
        uart_rx,
        uart_tx,
        RESOURCES.init(Resources::new()),
        MyCelullarConfig {
            reset_pin: None,
            power_pin: None,
            vint_pin: None,
        },
    );

    defmt::unwrap!(spawner.spawn(cell_task(runner)));

    Timer::after(Duration::from_millis(1000)).await;
    loop {
        control
            .set_desired_state(OperationState::DataEstablished)
            .await;
        info!("set_desired_state(PowerState::Alive)");
        while control.power_state() != OperationState::DataEstablished {
            Timer::after(Duration::from_millis(1000)).await;
        }
        Timer::after(Duration::from_millis(10000)).await;

        loop {
            Timer::after(Duration::from_millis(1000)).await;
            let operator = control.get_operator().await;
            info!("{}", operator);
            let signal_quality = control.get_signal_quality().await;
            info!("{}", signal_quality);
            if signal_quality.is_err() {
                let desired_state = control.desired_state();
                control.set_desired_state(desired_state).await
            }
            if let Ok(sq) = signal_quality {
                if let Ok(op) = operator {
                    if op.oper.is_none() {
                        continue;
                    }
                }
                if sq.rxlev > 0 && sq.rsrp != 255 {
                    break;
                }
            }
        }
        let dns = control
            .send(&ublox_cellular::command::dns::ResolveNameIp {
                resolution_type:
                    ublox_cellular::command::dns::types::ResolutionType::DomainNameToIp,
                ip_domain_string: "www.google.com",
            })
            .await;
        debug!("dns: {:?}", dns);
        Timer::after(Duration::from_millis(10000)).await;
        control.set_desired_state(OperationState::PowerDown).await;
        info!("set_desired_state(PowerState::PowerDown)");
        while control.power_state() != OperationState::PowerDown {
            Timer::after(Duration::from_millis(1000)).await;
        }

        Timer::after(Duration::from_millis(5000)).await;
    }
}

#[embassy_executor::task]
async fn cell_task(
    mut runner: InternalRunner<
        'static,
        BufferedUartRx<'static, USART3>,
        BufferedUartTx<'static, USART3>,
        MyCelullarConfig,
        INGRESS_BUF_SIZE,
        URC_CAPACITY,
    >,
) -> ! {
    runner.run().await
}
