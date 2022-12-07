#![no_std]

extern crate core2;
extern crate stdio;
extern crate app_io;

#[macro_use] extern crate log;
extern crate logger;

extern crate spawn;
extern crate task;

#[macro_use] extern crate terminal_print;
extern crate alloc;
extern crate hpet;
extern crate smoltcp_helper;

use core2::io::Write;

use log::Level;

use hpet::get_hpet;

use alloc::vec::Vec;
use alloc::string::String;
//use alloc::string::ToString;
use alloc::sync::Arc;

use smoltcp_helper::{millis_since};

// NOTE: the default params for theseus on QEMU gives us 4 cores

pub fn main(_args: Vec<String>) -> isize {
    let start_ticks = match get_hpet().as_ref().ok_or("couldn't get hpet timer") {
            Ok (time) => time.get_counter(),
            Err (_) => { println!("couldn't get hpet timer"); return -1; },
    } as u64;

    
    const STEP: i32 = 10_000_000;
    
    // disable logging for performance
    logger::set_log_level(Level::Error);
    
    // we need to give stdout to child tasks
    let stdout: Arc<stdio::StdioWriter> = Arc::new(app_io::stdout().expect("failed to get stdout"));
    //let mut stdout2<'static> = stdout.clone();

    println!("counting...");
    let mut total: i32 = 1;
    while total < 100_000_000 {
        
        let start1 = total;
        let end1 = if total + STEP > 100_000_000 { 100_000_000 } else { total + STEP };
        let start2 = if total + STEP > 100_000_000 { 100_000_000 } else { total + STEP };
        let end2 = if total + STEP*2 > 100_000_000 { 100_000_000 } else { total + STEP*2 };
        
        // TODO: instead of this, rerun same task but change params
        let task1 = match spawn::new_task_builder(count_between, (start1, end1, stdout.clone())).spawn() {
            Ok(task) => task,
            Err(errstr) => { println!("failed to spawn task: {}", errstr); return -1; }  
        };
        let task2 = match spawn::new_task_builder(count_between, (start2, end2, stdout.clone())).spawn() {
            Ok(task) => task,
            Err(errstr) => { println!("failed to spawn task: {}", errstr); return -1; }  
        };
            
        // wait for tasks to complete 
        task1.join().expect("failed to join task");
        total += match task1.take_exit_value().expect("task had no exit value") {
            task::ExitValue::Completed(ret_val) => *ret_val.downcast_ref::<i32>().expect("bad task return type"),
            task::ExitValue::Killed(kill_reason) => { println!("error occurred during a task. reason: {:?}", kill_reason); return -1; }
        };
        task2.join().expect("failed to join task");
        total += match task2.take_exit_value().expect("task had no exit value") {
            task::ExitValue::Completed(ret_val) => *ret_val.downcast_ref::<i32>().expect("bad task return type"),
            task::ExitValue::Killed(kill_reason) => { println!("error occurred during a task. reason: {:?}", kill_reason); return -1; }
        };
    }
    println!("done, {}", total);
    
    // this resets the logging level globally
    logger::set_log_level(Level::Trace);

    match millis_since(start_ticks) {
        Ok(time) => println!("time elapsed: {} ms", time),
        Err(err) => println!("couldn't get time at start: {}", err),
    };

    return 0;
}

// this function counts from start to end & returns the number counted
fn count_between((start, end, stdout): (i32, i32, Arc<stdio::StdioWriter>)) -> i32 {
    let mut i = 0;
    while i < (end - start) {
        if (start + i) % 50_000_000 == 0 {
            // must use stdout for child tasks instead of println
            let mut stdout_locked = stdout.lock();
            stdout_locked.write("halfway there...\n".as_bytes()).expect("failed write");
        }
        i += 1;
    }
    
    i
}
