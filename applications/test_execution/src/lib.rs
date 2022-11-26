#![no_std]

#[macro_use]
extern crate terminal_print;
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use simple_executor::SimpleExecutor;
use task_async::TaskAsync;

pub fn main(_args: Vec<String>) -> isize {
    let mut executor = SimpleExecutor::new();
    executor.spawn(TaskAsync::new(example_task()));
    executor.run();

    println!("after run");

    0
}

async fn async_number() -> u32 {
    if let Err(e) = sleep::sleep(1000) {
        println!("Error: {:?}", e);
    }
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}
