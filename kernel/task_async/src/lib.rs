#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use core::task::{Context, Poll};
use core::{future::Future, pin::Pin};

pub struct TaskAsync {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl TaskAsync {
    pub fn new(future: impl Future<Output = ()> + 'static) -> TaskAsync {
        TaskAsync {
            future: Box::pin(future),
        }
    }

    // Note Phillip made this a private method
    // I had to make it public so it could be accessed from the simple executor
    pub fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}
