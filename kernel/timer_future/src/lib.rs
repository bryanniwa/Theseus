#![no_std]

extern crate alloc;
extern crate sleep;
extern crate spawn;
extern crate task;
#[macro_use]
extern crate terminal_print;

use alloc::sync::Arc;
use core::task::{Context, Poll, Waker};
use core::{future::Future, pin::Pin};
use spin::Mutex;

pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

struct SharedState {
    completed: bool,
    waker: Option<Waker>,
}

impl Future for TimerFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut shared_state = self.shared_state.lock();
        if shared_state.completed {
            Poll::Ready(())
        } else {
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

struct TimerArgs {
    duration: usize,
    shared_state: Arc<Mutex<SharedState>>,
}

fn timer(args: TimerArgs) {
    if let Err(e) = sleep::sleep(args.duration) {
        println!("Error sleeping: {:?}", e);
    }
    let mut shared_state = args.shared_state.lock();
    shared_state.completed = true;
    if let Some(waker) = shared_state.waker.take() {
        waker.wake()
    }
}

impl TimerFuture {
    pub fn new(duration: usize) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState {
            completed: false,
            waker: None,
        }));

        let args = TimerArgs {
            duration,
            shared_state: shared_state.clone(),
        };

        spawn::new_task_builder(timer, args);

        TimerFuture { shared_state }
    }
}
