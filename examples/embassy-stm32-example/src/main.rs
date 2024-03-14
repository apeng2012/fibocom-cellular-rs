#![no_std]
#![no_main]
#![allow(stable_features)]
#![feature(type_alias_impl_trait)]

use atat::asynch::Client;
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, Level, Output, Speed};
use embassy_stm32::peripherals::USART3;
use embassy_stm32::time::Hertz;
use embassy_stm32::usart::{BufferedUart, BufferedUartRx, BufferedUartTx};
use embassy_stm32::{bind_interrupts, peripherals, Config};
use embassy_time::{Duration, Timer};
use no_std_net::Ipv4Addr;
use static_cell::make_static;
use ublox_cellular::asynch::state::OperationState;
use ublox_cellular::asynch::ublox_stack::tcp::TcpSocket;
use ublox_cellular::asynch::ublox_stack::{StackResources, UbloxStack};
use ublox_cellular::asynch::InternalRunner;
use ublox_cellular::asynch::Resources;
use ublox_cellular::config::{Apn, CellularConfig};
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
    power_pin: Option<Output<'static>>,
    // power_pin: Option<NoPin>,
    vint_pin: Option<Input<'static>>,
    // vint_pin: Option<NoPin>
}

impl<'a> CellularConfig<'a> for MyCelullarConfig {
    type ResetPin = Output<'static>;
    // type ResetPin = NoPin;
    type PowerPin = Output<'static>;
    // type PowerPin = NoPin;
    type VintPin = Input<'static>;
    // type VintPin = NoPin;

    const FLOW_CONTROL: bool = false;
    const HEX_MODE: bool = true;
    const APN: Apn<'a> = Apn::Given {
        name: "CMNET",
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
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;

        config.rcc.hse = Some(Hse {
            freq: Hertz(8_000_000),
            mode: HseMode::Oscillator,
        });
        // PLL uses HSE as the clock source
        config.rcc.pll = Some(Pll {
            src: PllSource::HSE,
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL9,
        });
        // System clock comes from PLL (= the 72 MHz main PLL output)
        config.rcc.sys = Sysclk::PLL1_P;
        // 72 MHz / 2 = 36 MHz APB1 frequency
        config.rcc.apb1_pre = APBPrescaler::DIV2;
        // 72 MHz / 1 = 72 MHz APB2 frequency
        config.rcc.apb2_pre = APBPrescaler::DIV1;
    }
    let p = embassy_stm32::init(config);

    let mut uart_config = embassy_stm32::usart::Config::default();
    {
        uart_config.baudrate = 115200;
        uart_config.parity = embassy_stm32::usart::Parity::ParityNone;
        uart_config.stop_bits = embassy_stm32::usart::StopBits::STOP1;
        uart_config.data_bits = embassy_stm32::usart::DataBits::DataBits8;
    }

    let tx_buf = &mut make_static!([0u8; 64])[..];
    let rx_buf = &mut make_static!([0u8; 64])[..];

    let cell_uart =
        BufferedUart::new(p.USART3, Irqs, p.PB11, p.PB10, tx_buf, rx_buf, uart_config).unwrap();
    let (uart_tx, uart_rx) = cell_uart.split();

    let resources = make_static!(Resources::<
        BufferedUartTx<USART3>,
        CMD_BUF_SIZE,
        INGRESS_BUF_SIZE,
        URC_CAPACITY,
    >::new());

    let (net_device, mut control, runner) = ublox_cellular::asynch::new_internal(
        uart_rx,
        uart_tx,
        resources,
        MyCelullarConfig {
            reset_pin: Some(Output::new(p.PA4, Level::Low, Speed::Low)),
            power_pin: Some(Output::new(p.PA5, Level::Low, Speed::Low)),
            vint_pin: None,
        },
    );

    // Init network stack
    let stack = &*make_static!(UbloxStack::new(
        net_device,
        make_static!(StackResources::<4>::new()),
    ));

    defmt::unwrap!(spawner.spawn(net_task(stack)));
    defmt::unwrap!(spawner.spawn(cell_task(runner)));

    Timer::after(Duration::from_millis(1000)).await;
    loop {
        control
            .set_desired_state(OperationState::DataEstablished)
            .await;
        info!("set_desired_state(PowerState::Alive)");
        let mut timeout_cnt = 0;
        while control.power_state() != OperationState::DataEstablished {
            Timer::after(Duration::from_millis(1000)).await;
            timeout_cnt += 1;
            if timeout_cnt > 60 * 3 {
                timeout_cnt = 0;
                control.set_desired_state(OperationState::PowerDown).await;
                control
                    .set_desired_state(OperationState::DataEstablished)
                    .await;
            }
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
                if sq.rssi != 99 {
                    break;
                }
            }
        }
        test_visit_ifconfig(stack).await;

        Timer::after(Duration::from_millis(10000)).await;
        control.set_desired_state(OperationState::PowerDown).await;
        info!("set_desired_state(PowerState::PowerDown)");
        while control.power_state() != OperationState::PowerDown {
            Timer::after(Duration::from_millis(1000)).await;
        }

        Timer::after(Duration::from_millis(5000)).await;
    }
}

const RX_SIZE: usize = INGRESS_BUF_SIZE;

const RX_BUFFER_SIZE: usize = 1024;
const TX_BUFFER_SIZE: usize = 1024;
const SERVER_ADDRESS: Ipv4Addr = Ipv4Addr::new(172, 67, 199, 190);
const HTTP_PORT: u16 = 80;

async fn test_visit_ifconfig(
    stack: &'static UbloxStack<
        Client<'static, BufferedUartTx<'static, USART3>, INGRESS_BUF_SIZE>,
        URC_CAPACITY,
    >,
) {
    info!("Testing visit ifconfig.net...");

    let mut rx_buffer = [0; RX_BUFFER_SIZE];
    let mut tx_buffer = [0; TX_BUFFER_SIZE];
    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    // socket.set_timeout(Some(Duration::from_secs(10)));

    // info!(
    //     "connecting to {:?}:{}...",
    //     debug2Format(&SERVER_ADDRESS),
    //     HTTP_PORT
    // );
    if let Err(e) = socket.connect((SERVER_ADDRESS, HTTP_PORT)).await {
        error!("connect error: {:?}", e);
        return;
    }

    info!("Sending HTTP request...");
    let request = b"GET / HTTP/1.1\r\nAccept: text/plain\r\nHost: ifconfig.net\r\n\r\n\x1A";
    let bytes_write = socket
        .write(request)
        .await
        .expect("Could not send HTTP request");
    info!("Write {} bytes", bytes_write);

    let mut rx_buf = [0; RX_SIZE];
    let bytes_read = socket
        .read(&mut rx_buf)
        .await
        .expect("Error while receiving data");
    info!("Read {} bytes", bytes_read);

    Timer::after(Duration::from_millis(10000)).await;
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

#[embassy_executor::task]
async fn net_task(
    stack: &'static UbloxStack<
        Client<'static, BufferedUartTx<'static, USART3>, INGRESS_BUF_SIZE>,
        URC_CAPACITY,
    >,
) -> ! {
    stack.run().await
}
