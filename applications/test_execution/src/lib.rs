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
    executor.spawn(TaskAsync::new(example_task(1, 1000)));
    println!("test message");
    executor.spawn(TaskAsync::new(example_task(2, 500)));

    executor.run();

    0
}

async fn async_number(sleep_time: usize, task_num: u8) -> u8 {
    if let Err(e) = sleep::sleep(sleep_time) {
        println!("Error: {:?}", e);
    }
    task_num
}

async fn example_task(task_num: u8, sleep_time: usize) {
    let number = async_number(sleep_time, task_num);
    println!("Task {}: async number: {}", task_num, number.await);
}
