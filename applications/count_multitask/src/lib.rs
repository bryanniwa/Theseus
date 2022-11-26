#![no_std]

extern crate log;
extern crate logger;

extern crate spawn;
extern crate task;

#[macro_use] extern crate terminal_print;
extern crate alloc;
extern crate hpet;
extern crate smoltcp_helper;

use log::Level;

use hpet::get_hpet;

use alloc::vec::Vec;
use alloc::string::String;
//use alloc::string::ToString;
//use alloc::sync::Arc;

use smoltcp_helper::{millis_since};

// NOTE: the default params for theseus on QEMU gives us 4 cores

pub fn main(_args: Vec<String>) -> isize {
    let start_ticks = match get_hpet().as_ref().ok_or("couldn't get hpet timer") {
            Ok (time) => time.get_counter(),
            Err (_) => { println!("couldn't get hpet timer"); return -1; },
    };

    
    const STEP: i32 = 10_000_000;
    
    // disable logging for performance
   logger::set_log_level(Level::Error);

    println!("counting...");
    let mut total: i32 = 1;
    while total < 100_000_000 {
        
        let start1 = total;
        let end1 = if total + STEP > 100_000_000 { 100_000_000 } else { total + STEP };
        let start2 = if total + STEP > 100_000_000 { 100_000_000 } else { total + STEP };
        let end2 = if total + STEP*2 > 100_000_000 { 100_000_000 } else { total + STEP*2 };
        
        // TODO: instead of this, rerun same task but change params
        let task1 = match spawn::new_task_builder(count_between, (start1, end1)).spawn() {
            Ok(task) => task,
            Err(errstr) => { println!("failed to spawn task: {}", errstr); return -1; }  
        };
        let task2 = match spawn::new_task_builder(count_between, (start2, end2)).spawn() {
            Ok(task) => task,
            Err(errstr) => { println!("failed to spawn task: {}", errstr); return -1; }  
        };
            
        // wait for tasks to complete 
        task1.join().expect("failed to join task");
        total += match task1.take_exit_value().expect("task had no exit value") {
            task::ExitValue::Completed(ret_val) => *ret_val.downcast_ref::<i32>().expect("bad task return type"),
            _ => { println!("error occurred during a task"); return -1; }
        };
        task2.join().expect("failed to join task");
        total += match task2.take_exit_value().expect("task had no exit value") {
            task::ExitValue::Completed(ret_val) => *ret_val.downcast_ref::<i32>().expect("bad task return type"),
            _ => { println!("error occurred during a task"); return -1; }
        };

    }
    println!("done, {}", total);

    logger::set_log_level(Level::Trace);

    match millis_since(start_ticks) {
        Ok(time) => println!("time elapsed: {} ms", time),
        Err(err) => println!("couldn't get time at start: {}", err),
    };

    0
}

// this function counts from start to end & returns the number counted
fn count_between((start, end): (i32, i32)) -> i32 {
    //println!("task doing {}", start);
    let mut i = 0;
    while i < (end - start) {
        if (start + i) % 50_000_000 == 0 {
            // may not be exactly halfway because multitasking
            println!("halfway there...");
        }
        i += 1;
    }
    
    i
}
