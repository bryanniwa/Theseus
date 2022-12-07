#![no_std]

extern crate alloc;
extern crate task_async;

use alloc::collections::VecDeque;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use task_async::TaskAsync;

pub struct SimpleExecutor {
    task_queue: VecDeque<TaskAsync>,
}

impl SimpleExecutor {
    pub fn new() -> SimpleExecutor {
        SimpleExecutor {
            task_queue: VecDeque::new(),
        }
    }

    pub fn spawn(&mut self, task: TaskAsync) {
        self.task_queue.push_back(task)
    }

    pub fn run(&mut self) {
        while let Some(mut task) = self.task_queue.pop_front() {
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {} // task done
                // if the task isn't done yet then put it back in the queue
                Poll::Pending => self.task_queue.push_back(task),
            }
        }
    }
}

fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}

fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    #[allow(clippy::zero_ptr)]
    RawWaker::new(0 as *const (), vtable)
}
