#![no_std]

extern crate core2;
extern crate stdio;
extern crate app_io;

extern crate log;
extern crate logger;

extern crate spin;
//extern crate spawn;
//extern crate task;

#[macro_use] extern crate terminal_print;
extern crate alloc;
extern crate hpet;
extern crate smoltcp_helper;

extern crate task_async;

use core2::io::Write;

use log::Level;

use hpet::get_hpet;

use alloc::vec::Vec;
use alloc::string::String;
//use alloc::string::ToString;
use alloc::sync::Arc;
use spin::Mutex;

use smoltcp_helper::{millis_since};

use simple_executor::SimpleExecutor;
use task_async::TaskAsync;

// NOTE: the default params for theseus on QEMU gives us 4 cores

pub fn main(_args: Vec<String>) -> isize {
    let start_ticks = match get_hpet().as_ref().ok_or("couldn't get hpet timer") {
            Ok (time) => time.get_counter(),
            Err (_) => { println!("couldn't get hpet timer"); return -1; },
    } as u64;

    const STEP: i32 = 10_000_000;
    
    // disable logging for performance
    logger::set_log_level(Level::Error);
    
    let mut executor = SimpleExecutor::new();

    // we need to give stdout to child tasks
    let stdout: Arc<stdio::StdioWriter> = Arc::new(app_io::stdout().expect("failed to get stdout"));
    
    println!("counting...");
    let mut total: Arc<Mutex<i32>> = Arc::new(Mutex::new(1));
    let mut total_value = 1;
    
    while total_value < 100_000_000 {
        
        // load 4 tasks into the executor
        let mut tmp_total_value = total_value;
        for i in 0..4 {
            let start1 = tmp_total_value;
            let end1 = if tmp_total_value + STEP > 100_000_000 { 100_000_000 } else { tmp_total_value + STEP };
            let input_tup = (start1, end1, stdout.clone(), total.clone());
            executor.spawn(
                TaskAsync::new(async {
                    count_between(input_tup);
                })
            );
            
            tmp_total_value += STEP;
        }
        
        // run tasks in serial
        executor.run();
        
        {
            // update readable total so we can loop 
            total_value = (*total.lock());
        }
    }

    println!("done, {}", total.lock());
    
    // this resets the logging level globally
    logger::set_log_level(Level::Trace);

    match millis_since(start_ticks) {
        Ok(time) => println!("time elapsed: {} ms", time),
        Err(err) => println!("couldn't get time at start: {}", err),
    };

    return 0;
}

// this function counts from start to end & returns the number counted
fn count_between((start, end, stdout, output): (i32, i32, Arc<stdio::StdioWriter>, Arc<Mutex<i32>>)) -> i32 {
    let mut i = 0;
    while i < (end - start) {
        if (start + i) % 50_000_000 == 0 {
            // must use stdout for child tasks instead of println
            let mut stdout_locked = stdout.lock();
            stdout_locked.write("halfway there...\n".as_bytes()).expect("failed write");
        }
        i += 1;
    }
    
    {
        *(output.lock()) += i;
    }

    0
}
