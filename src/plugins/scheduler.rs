use std::boxed::Box;
use std::error::Error;
use std::fmt;
use std::thread;
use std::time::{Duration, Instant};

use log;
use redis;

use crate::models::database::Query;
use crate::models::Peripheral;
use crate::plugins::Plugin;

/// A single task to be executed by the scheduler.
pub struct Task {
    /// A short description of the task that will appear in the logs.
    description: String,

    /// The minimum amount of time that must elapse between the end of one run of the task and the
    /// start of the next run.
    interval: Duration,

    /// The time at which the previous run of the task finished.
    last_tick: Instant,

    /// The function that will be executed by the scheduler.
    ///
    /// The callback should not return anything, including error information. Instead, it or the
    /// functions that it calls should log any errors. This keeps the scheduler's runtime loop
    /// simple and helps ensure that tasks that cause an error do not interfere with the rest of
    /// scheduler's operation.
    callback: Box<dyn Fn(&mut Peripheral, &Plugin)>,
}

impl Task {
    pub fn new(
        description: String,
        interval: Duration,
        last_tick: Instant,
        callback: Box<dyn Fn(&mut Peripheral, &Plugin)>,
    ) -> Task {
        Task {
            description,
            interval,
            last_tick,
            callback,
        }
    }
}

// A Task instance is created outside of the only thread in which it is used. Implementing Send
// allows us to move the task into the thread after it is created.
unsafe impl Send for Task {}

pub struct Scheduler {
    pub plugin: Plugin,

    db: redis::Connection,
    peripheral: Peripheral,

    /// The time between the end of one run of the scheduler and the start of the next.
    sleep: Duration,
    tasks: Vec<Task>,
}

impl Scheduler {
    pub fn new(
        plugin: Plugin,
        db: redis::Connection,
        peripheral: Peripheral,
        sleep: Duration,
    ) -> Scheduler {
        let tasks: Vec<Task> = Vec::new();

        Scheduler {
            plugin,
            db,
            peripheral,
            sleep,
            tasks,
        }
    }

    pub fn push(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn run(mut scheduler: Scheduler) {
        thread::spawn(move || -> Result<(), SchedulerRuntimeError> {
            log::info!("Spawning new thread for plugin: {:?}", scheduler.plugin);

            let mut now: Instant;
            let mut count: usize;
            loop {
                log::debug!(
                    "Checking scheduled tasks for peripheral {}",
                    scheduler.peripheral.id()
                );

                count = 0;
                for task in &mut scheduler.tasks {
                    now = Instant::now();

                    if now.duration_since(task.last_tick) > task.interval {
                        log::debug!("Executing task: {}", task.description);
                        (task.callback)(&mut scheduler.peripheral, &scheduler.plugin);

                        task.last_tick = Instant::now();
                        count += 1;
                    }
                }

                // Only update the database entry if something happened.
                if count != 0 {
                    log::debug!(
                        "Updating database entry for peripheral {}",
                        scheduler.peripheral.id()
                    );
                    match scheduler.peripheral.set(&scheduler.db) {
                        Ok(_) => (),
                        Err(e) => log::error!("{}", e),
                    };
                }

                thread::sleep(scheduler.sleep);
            }
        });
    }
}
#[derive(Debug)]
pub struct SchedulerRuntimeError {}

impl Error for SchedulerRuntimeError {
    fn description(&self) -> &str {
        "The scheduler thread failed"
    }
}

impl fmt::Display for SchedulerRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The scheduler thread failed")
    }
}
