#![no_std]

extern crate log;
extern crate logger;

#[macro_use] extern crate terminal_print;
extern crate alloc;
extern crate hpet;
extern crate smoltcp_helper;

use hpet::get_hpet;

use alloc::vec::Vec;
use alloc::string::String;
//use alloc::string::ToString;
//use alloc::sync::Arc;

use smoltcp_helper::{millis_since};

use log::Level;

pub fn main(_args: Vec<String>) -> isize {
    let start_ticks = match get_hpet().as_ref().ok_or("couldn't get hpet timer") {
            Ok (time) => time.get_counter(),
            Err (_) => { println!("couldn't get hpet timer"); return -1; },
    } as u64;

    // disable logging for performance
    logger::set_log_level(Level::Error);

    println!("counting...");
    let mut i = 1;
    while i < 100_000_000 {
        if i % 50_000_000 == 0 {
            println!("halfway there...");
        }
        i += 1;
    }
    println!("done, {}", i);
    
    // this resets the logging level globally
    logger::set_log_level(Level::Trace);

    match millis_since(start_ticks) {
        Ok(time) => println!("time elapsed: {} ms", time),
        Err(err) => println!("couldn't get time at start: {}", err),
    };

    return 0;
}