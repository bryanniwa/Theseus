#![no_std]

extern crate task_async;
#[macro_use]
extern crate terminal_print;
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use simple_executor::SimpleExecutor;
use task_async::TaskAsync;
use timer_future::TimerFuture;

pub fn main(_args: Vec<String>) -> isize {
    let mut executor = SimpleExecutor::new();
    executor.spawn(TaskAsync::new(async {
        for _ in 0..3 {
            println!("howdy 1!");
            TimerFuture::new(2000).await;
            println!("done 1!")
        }
    }));
    println!("test message");
    executor.spawn(TaskAsync::new(async {
        for _ in 0..3 {
            println!("howdy 2!");
            TimerFuture::new(500).await;
            println!("done 2!")
        }
    }));
    executor.run();

    0
}
