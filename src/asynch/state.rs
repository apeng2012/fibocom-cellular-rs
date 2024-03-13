#![allow(dead_code)]

use core::cell::RefCell;
use core::task::Context;

use atat::asynch::AtatClient;
use atat::UrcSubscription;
use embassy_sync::blocking_mutex::raw::NoopRawMutex;
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::pubsub::PubSubChannel;
use embassy_sync::waitqueue::WakerRegistration;

const MAX_STATE_LISTENERS: usize = 5;

/// The link state of a network device.
#[derive(PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LinkState {
    /// The link is down.
    Down,
    /// The link is up.
    Up,
}

/// If the celular modem is up and responding to AT.
#[derive(PartialEq, Eq, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum OperationState {
    PowerDown = 0,
    PowerUp,
    Alive,
    Initialized,
    Connected,
    #[cfg(not(feature = "ppp"))]
    DataEstablished,
}

impl TryFrom<isize> for OperationState {
    fn try_from(state: isize) -> Result<Self, ()> {
        match state {
            0 => Ok(OperationState::PowerDown),
            1 => Ok(OperationState::PowerUp),
            2 => Ok(OperationState::Alive),
            3 => Ok(OperationState::Initialized),
            4 => Ok(OperationState::Connected),
            #[cfg(not(feature = "ppp"))]
            5 => Ok(OperationState::DataEstablished),
            _ => Err(()),
        }
    }
    type Error = ();
}

use crate::command::Urc;
use crate::error::Error;

use super::AtHandle;

pub struct State {
    shared: Mutex<NoopRawMutex, RefCell<Shared>>,
    desired_state_pub_sub: PubSubChannel<NoopRawMutex, OperationState, 1, MAX_STATE_LISTENERS, 1>,
}

impl State {
    pub const fn new() -> Self {
        Self {
            shared: Mutex::new(RefCell::new(Shared {
                link_state: None,
                power_state: OperationState::PowerDown,
                desired_state: OperationState::PowerDown,
                waker: WakerRegistration::new(),
            })),
            desired_state_pub_sub: PubSubChannel::<
                NoopRawMutex,
                OperationState,
                1,
                MAX_STATE_LISTENERS,
                1,
            >::new(),
        }
    }
}

/// State of the LinkState
pub struct Shared {
    link_state: Option<LinkState>,
    power_state: OperationState,
    desired_state: OperationState,
    waker: WakerRegistration,
}

pub struct Runner<'d> {
    pub(crate) shared: &'d Mutex<NoopRawMutex, RefCell<Shared>>,
    pub(crate) desired_state_pub_sub:
        &'d PubSubChannel<NoopRawMutex, OperationState, 1, MAX_STATE_LISTENERS, 1>,
}

#[derive(Clone, Copy)]
pub struct StateRunner<'d> {
    shared: &'d Mutex<NoopRawMutex, RefCell<Shared>>,
    desired_state_pub_sub:
        &'d PubSubChannel<NoopRawMutex, OperationState, 1, MAX_STATE_LISTENERS, 1>,
}

impl<'d> Runner<'d> {
    pub fn state_runner(&self) -> StateRunner<'d> {
        StateRunner {
            shared: self.shared,
            desired_state_pub_sub: self.desired_state_pub_sub,
        }
    }

    pub fn set_link_state(&mut self, state: Option<LinkState>) {
        self.shared.lock(|s| {
            let s = &mut *s.borrow_mut();
            s.link_state = state;
            s.waker.wake();
        });
    }

    pub fn set_power_state(&mut self, state: OperationState) {
        self.shared.lock(|s| {
            let s = &mut *s.borrow_mut();
            s.power_state = state;
            s.waker.wake();
        });
    }

    pub fn set_desired_state(&mut self, ps: OperationState) {
        self.shared.lock(|s| {
            let s = &mut *s.borrow_mut();
            s.desired_state = ps;
            s.waker.wake();
        });
        self.desired_state_pub_sub
            .immediate_publisher()
            .publish_immediate(ps);
    }
}

impl<'d> StateRunner<'d> {
    pub fn set_link_state(&self, state: Option<LinkState>) {
        self.shared.lock(|s| {
            let s = &mut *s.borrow_mut();
            s.link_state = state;
            s.waker.wake();
        });
    }

    pub fn link_state_poll_fn(&mut self, cx: &mut Context) -> Option<LinkState> {
        self.shared.lock(|s| {
            let s = &mut *s.borrow_mut();
            s.waker.register(cx.waker());
            s.link_state
        })
    }

    pub fn set_power_state(&self, state: OperationState) {
        self.shared.lock(|s| {
            let s = &mut *s.borrow_mut();
            s.power_state = state;
            s.waker.wake();
        });
    }

    pub fn power_state_poll_fn(&mut self, cx: &mut Context) -> OperationState {
        self.shared.lock(|s| {
            let s = &mut *s.borrow_mut();
            s.waker.register(cx.waker());
            s.power_state
        })
    }

    pub fn link_state(&mut self) -> Option<LinkState> {
        self.shared.lock(|s| {
            let s = &mut *s.borrow_mut();
            s.link_state
        })
    }

    pub fn power_state(&mut self) -> OperationState {
        self.shared.lock(|s| {
            let s = &mut *s.borrow_mut();
            s.power_state
        })
    }

    pub fn desired_state(&mut self) -> OperationState {
        self.shared.lock(|s| {
            let s = &mut *s.borrow_mut();
            s.desired_state
        })
    }

    pub async fn set_desired_state(&mut self, ps: OperationState) {
        self.shared.lock(|s| {
            let s = &mut *s.borrow_mut();
            s.desired_state = ps;
            s.waker.wake();
        });
        self.desired_state_pub_sub
            .immediate_publisher()
            .publish_immediate(ps);
    }

    pub async fn wait_for_desired_state(
        &mut self,
        ps: OperationState,
    ) -> Result<OperationState, Error> {
        if self.desired_state() == ps {
            info!("Desired state already set to {:?}, returning", ps);
            return Ok(ps);
        }
        let mut sub = self
            .desired_state_pub_sub
            .subscriber()
            .map_err(|x| Error::SubscriberOverflow(x))?;
        loop {
            let ps_now = sub.next_message_pure().await;
            if ps_now == ps {
                return Ok(ps_now);
            }
        }
    }

    pub async fn wait_for_desired_state_change(&mut self) -> Result<OperationState, Error> {
        let mut sub = self
            .desired_state_pub_sub
            .subscriber()
            .map_err(|x| Error::SubscriberOverflow(x))?;
        Ok(sub.next_message_pure().await)
    }
}

pub fn new<'d, AT: AtatClient, const URC_CAPACITY: usize>(
    state: &'d mut State,
    at: AtHandle<'d, AT>,
    urc_subscription: UrcSubscription<'d, Urc, URC_CAPACITY, 2>,
) -> (Runner<'d>, Device<'d, AT, URC_CAPACITY>) {
    let runner = Runner {
        shared: &state.shared,
        desired_state_pub_sub: &state.desired_state_pub_sub,
    };

    let desired_state_pub_sub = runner.desired_state_pub_sub;

    (
        runner,
        Device {
            shared: TestShared {
                inner: &state.shared,
            },
            urc_subscription,
            at,
            desired_state_pub_sub,
        },
    )
}

pub fn new_ppp<'d>(state: &'d mut State) -> Runner<'d> {
    Runner {
        shared: &state.shared,
        desired_state_pub_sub: &state.desired_state_pub_sub,
    }
}

pub struct Device<'d, AT: AtatClient, const URC_CAPACITY: usize> {
    pub(crate) shared: TestShared<'d>,
    pub(crate) desired_state_pub_sub:
        &'d PubSubChannel<NoopRawMutex, OperationState, 1, MAX_STATE_LISTENERS, 1>,
    pub(crate) at: AtHandle<'d, AT>,
    pub(crate) urc_subscription: UrcSubscription<'d, Urc, URC_CAPACITY, 2>,
}

impl<'d, AT: AtatClient, const URC_CAPACITY: usize> Device<'d, AT, URC_CAPACITY> {
    pub fn desired_state(&mut self) -> OperationState {
        self.shared.desired_state()
    }

    pub async fn wait_for_desired_state(
        &mut self,
        ps: OperationState,
    ) -> Result<OperationState, Error> {
        if self.desired_state() == ps {
            return Ok(ps);
        }
        let mut sub = self
            .desired_state_pub_sub
            .subscriber()
            .map_err(|x| Error::SubscriberOverflow(x))?;
        loop {
            let ps_now = sub.next_message_pure().await;
            if ps_now == ps {
                return Ok(ps_now);
            }
        }
    }

    pub async fn dummy_run(&mut self) -> ! {
        loop {
            self.urc_subscription.next_message_pure().await;
        }
    }
}

pub struct TestShared<'d> {
    inner: &'d Mutex<NoopRawMutex, RefCell<Shared>>,
}

impl<'d> TestShared<'d> {
    pub fn link_state(&mut self, cx: &mut Context) -> Option<LinkState> {
        self.inner.lock(|s| {
            let s = &mut *s.borrow_mut();
            s.waker.register(cx.waker());
            s.link_state
        })
    }

    pub fn desired_state(&mut self) -> OperationState {
        self.inner.lock(|s| {
            let s = &mut *s.borrow_mut();
            s.desired_state
        })
    }
}
