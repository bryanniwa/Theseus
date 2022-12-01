#![no_std]
#[macro_use] extern crate terminal_print;

extern crate alloc;
extern crate task;
extern crate getopts;
extern crate fs_node;
extern crate vfs_node;

use alloc::vec::Vec;
use alloc::string::String;
use alloc::string::ToString;
use memfs::MemFile;

pub fn main(args: Vec<String>) -> isize {
    if args.is_empty() {
        println!("Error: missing arguments");
        return -1;
    }

    let Ok(curr_wd) = task::with_current_task(|t| t.get_env().lock().working_dir.clone()) else {
        println!("failed to get current task");
        return -1;
    };

    let mut ret = 0;

    for file_name in args.iter() {
        // add file to current directory
        if let Err(err) = MemFile::new(file_name.to_string(), &curr_wd) {
            println!("Error creating {:?}: {}", file_name, err);
            ret = -1;
        }
        println!("{:?} created.", file_name);
    }

    ret
}