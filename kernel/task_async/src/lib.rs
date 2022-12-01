#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll},
};

pub struct TaskAsync {
    id: TaskId,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl TaskAsync {
    pub fn new(future: impl Future<Output = ()> + 'static) -> TaskAsync {
        TaskAsync {
            id: TaskId::new(),
            future:Box::pin(future),
        }
    }

    // Note Phillip made this a private method
    // I had to make it public so it could be accessed from the simple executor
    pub fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);

impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}