#![no_std]

#[macro_use]
extern crate terminal_print;
extern crate alloc;
extern crate crossbeam_queue;
extern crate task_async;
extern crate x86_64;

use alloc::{collections::BTreeMap, sync::Arc, task::Wake, vec::Vec};
use core::task::{Context, Poll, Waker};
use core::{future::Future, pin::Pin};
use crossbeam_queue::ArrayQueue;
use task_async::{TaskAsync, TaskId};
use spin::Mutex;

use task::JoinableTaskRef;

pub struct WakerExecutor {
    // tasks are accessed by TaskId in the map
    tasks: BTreeMap<TaskId, TaskAsync>,
    // wrapped into Arc type that implements reference counting
    task_queue: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl WakerExecutor {
    pub fn new() -> WakerExecutor {
        WakerExecutor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
        }
    }

    // push the corresponding async task in the queue when spawned
    pub fn spawn(&mut self, task: TaskAsync) {
        let task_id = task.id;

        // check if task ID exists and if the queue is full
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with the same ID already in tasks");
        }
        self.task_queue.push(task_id).expect("queue full");
    }

    pub fn run(&mut self) -> Result<(), ()> {
        loop {
            self.run_ready_tasks();
            if self.tasks.is_empty() {
                return Ok(());
            }
            self.sleep_if_idle();
        }
    }

    fn run_ready_tasks(&mut self) {
        // destructure `self` to avoid borrow checker errors
        let Self {
            tasks,
            task_queue,
            waker_cache,
        } = self;

        // loop over all tasks in the task queue
        while let Some(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => {
                    println!("Task ID: {:?}", task_id);
                    task
                },
                None => continue, // task doesn't exist anymore
            };

            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    // task is done so remove it and its cached waker
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }

    fn sleep_if_idle(&self) {
        use x86_64::instructions::interrupts::{self, enable_and_hlt};

        interrupts::disable();
        if self.task_queue.is_empty() {
            enable_and_hlt();
        } else {
            interrupts::enable();
        }
    }
}

// waker implementation
struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task_queue full");
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}

// --------------------------
// join all implementation attempt:

/*
// TODO: make this function accept any iterator
fn join_all(futures: Vec<dyn Future<Output = ()> + Send>) -> JoinAll
{
    JoinAll::new(futures)
}

struct JoinAll {
    futures: Option<Vec<dyn Future<Output = ()> + Send>>,
    taskref_list: Vec<JoinableTaskRef>,
    shared_state: Arc<Mutex<JoinAllSharedState>>,
}

struct JoinAllSharedState {
    pub num_left: u32, // the number of futures not yet done
    pub waker: Option<Waker>,
}

struct JoinAllBodyArgs {
    pub future: dyn Future<Output = ()> + Send,
    pub shared_state: Arc<Mutex<JoinAllSharedState>>,
}

impl JoinAll {
    fn new(futures: Vec<dyn Future<Output = ()> + Send>) -> JoinAll {
        let len = futures.len() as u32;
        JoinAll { 
            futures: Some(futures),
            taskref_list: Vec::new(),
            shared_state: Arc::new(Mutex::new(
                JoinAllSharedState {
                    num_left: len,
                    waker: None,
                }
            )),
        }
    }

    // this should usually be run on a new theseus task
    fn body(args: JoinAllBodyArgs) -> u32 {
        let mut executor = WakerExecutor::new();
        executor.spawn(
            TaskAsync::new(args.future)
        );
        if let Err(_err) = executor.run() {
            println!("The Executor raised an error!");
            return 1;
        }

        // we're done! check if we should wake the executor
        let mut num_left: u32 = 0;
        {
            let mut shared_state = args.shared_state.lock();
            shared_state.num_left -= 1;
            num_left = shared_state.num_left;
        }

        // only wake if this is the last task
        if num_left == 0 {
            let mut shared_state = args.shared_state.lock();
            if let Some(thewaker) = &shared_state.waker {
                thewaker.wake_by_ref();
            }
            // TODO: add an error in the else condition
        }

        0 // for success
    } 
}

// now join_all can be awaited
impl Future for JoinAll {
    type Output = (); // TODO: support return values
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match &self.futures {
            Some(futures) => {
                // do initial poll for all futures & give them their own threads
                for f in futures {
                    match spawn::new_task_builder(Self::body, 
                        JoinAllBodyArgs { 
                            future: f,
                            shared_state: self.shared_state.clone(),
                        }
                    ).spawn() {
                        Ok(taskref) => self.taskref_list.push(taskref),
                        Err(_err) => println!("failed to start task!"),
                    };
                }
                self.futures = None;
                self.shared_state.lock().waker = Some(cx.waker().clone());

                Poll::Pending
            },
            None => {
                // check if all children tasks are done
                let mut num_left = 0;
                { num_left = self.shared_state.lock().num_left; }
                if num_left == 0 {
                    // clean up threads
                    for taskref in &self.taskref_list {
                        let _ = taskref.join();
                    }

                    Poll::Ready(())
                } else {
                    // NOTE: this is probably unreachable
                    self.shared_state.lock().waker = Some(cx.waker().clone());

                    Poll::Pending 
                }
            },
        }
    }
}
*/
