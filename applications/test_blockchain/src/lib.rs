#![no_std]

extern crate spawn;
extern crate task;
//extern crate task_async;

#[macro_use] extern crate terminal_print;
extern crate alloc;
#[macro_use] extern crate log;
extern crate logger;

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::sync::Arc;
use alloc::boxed::Box;

//use simple_executor::SimpleExecutor;
//use task_async::TaskAsync;

use sha3::digest::generic_array::GenericArray;
use sha3::{Digest, Sha3_256};
use digest::consts::U32;

use log::Level;

// NOTE: 
// this implementation is based on a CMPT383 assignment by Andres Miltner & Greg Baker (though this is significantly different)

type Hash = GenericArray<u8, U32>;

#[derive(Debug,Clone)]
struct Block {
    pub previous_hash: Hash,
    pub generation: u64,
    pub data: String,
    pub proof: Option<u64>, 
}

impl Block {
    const difficulty_bits: u8 = 8;

    pub fn first_block() -> Block {
        Self {
            previous_hash: Hash::default(),
            generation: 0,
            data: String::from(""),
            proof: None,
        }
    }

    pub fn new_block(previous: &Block, data: String) -> Block {
        Self {
            previous_hash: previous.hash_given(previous.proof.expect("invalid block in the chain!")),
            generation: previous.generation + 1,
            data,
            proof: None,
        }    
    }

    fn hash_str(&self, proof: u64) -> String {
        let mut hash_str = String::from("");
        //hash_str.push_str(&(self.previous_hash.to_string())); // TODO: display this somehow
        //hash_str.push_str(",");
        hash_str.push_str(&(self.generation.to_string()));
        hash_str.push_str(",");
        hash_str.push_str(&(self.data.to_string()));
        hash_str.push_str(",");
        hash_str.push_str(&(proof.to_string()));
        hash_str
    }

    fn hash_given(&self, proof: u64) -> Hash {
        let mut hasher = Sha3_256::new();
        let the_hash_str = self.hash_str(proof);

        hasher.update(&the_hash_str);
        hasher.finalize()
    }
    
    fn hash_is_valid(hash:Hash) -> bool {
        let n_bytes = Block::difficulty_bits / 8;
        let n_bits = Block::difficulty_bits % 8;
        for i in 0..n_bytes {
            if hash[hash.len()-1 - (i as usize)] != 0u8 {
                return false;
            }
        } 

        hash[hash.len()-1 - (n_bytes as usize)] % (1 << n_bits) == 0u8
    }
    
    fn mine_chunk((start, end, block): (u64, u64, Arc<Block>)) -> Option<u64> {
        for proof in start..end {
            if Block::hash_is_valid(block.hash_given(proof)) {
                return Some(proof);
            }
        }
        None
    }

    // NOTE: we want workers tasks all running at once. When all finish, another batch starts.
    // returns whether an error occurs or not
    pub fn mine(self: &mut Block, workers: usize) -> bool {
        let range_start: u64 = 0;
        let range_end: u64 = 8 * (1 << Block::difficulty_bits);
        let chunks: u64 = 2345;

        //let mut queue: Vec<Box<dyn Fn() -> Option<u64>>> = Vec::new();

        let block_ref = Arc::new(self.clone()); // a thread-read safe reusable copy of self

        // create all the runnable chunks
        let step = ((range_end - range_start) / chunks) + if (range_end - range_start) % chunks == 0 { 0 } else { 1 }; 
        let mut i: u64 = 0;
        //while i < chunks {
        //    let block_ref_clone = block_ref.clone();
        //    let block_start = u64::min(range_start + i * step, range_end);
        //    let block_end = u64::min(range_start + (i+1) * step, range_end);
        //    queue.push(Box::new(move || { Block::mine_chunk(block_start, block_end, &block_ref_clone) }));

        //    i += 1;
        //}

        // run the chunks in parallel
        // i = 0;
        let mut found_proof = false;
        while i < chunks || found_proof {
            // 0 < chunks - i <= workers
            let chunks_this_cycle = if i + (workers as u64) < chunks { workers as u64 } else { chunks - i };
            
            // create theseus tasks
            let mut current_workers = Vec::new();
            for i in 0..chunks_this_cycle {
                let block_start = u64::min(range_start + i * step, range_end);
                let block_end = u64::min(range_start + (i+1) * step, range_end);
                let task = match spawn::new_task_builder(Block::mine_chunk, (block_start, block_end, block_ref.clone())).spawn() {
                    Ok(thetask) => thetask,
                    Err(_) => { return false; }
                };
                current_workers.push(task);
            }

            // run theseus tasks
            for task in current_workers {
                task.join().expect("failed to join task... somehow");
                let maybe_proof = match task.take_exit_value().expect("task had no exit value") {
                    task::ExitValue::Completed(ret_val) => *ret_val.downcast_ref::<Option<u64>>().expect("bad task return type"),
                    task::ExitValue::Killed(_) => { return false; },
                };
                
                if let Some(proof) = maybe_proof {
                    found_proof = true;
                    self.proof = Some(proof);
                } 
            }

            i += workers as u64;
        }

        return true;
    }
}

// -------------------------

// NOTE: this is the preemptive version
pub fn main(_args: Vec<String>) -> isize {
    logger::set_log_level(Level::Error);
    
    let mut block_1 = Block::first_block();
    let _success_1 = block_1.mine(4);
    
    let mut block_2 = Block::new_block(&block_1, String::from("this first block is super cool"));
    let _success_2 = block_2.mine(4);

    let mut block_3 = Block::new_block(&block_2, String::from("here is some great data for the 2nd block.\nIt even has mutiple lines!"));
    let _success_3 = block_3.mine(4);

    println!("created a chain of 3 blocks!\nproofs:\t{:?}\t{:?}\t{:?}", block_1.proof, block_2.proof, block_3.proof);

    0
}

