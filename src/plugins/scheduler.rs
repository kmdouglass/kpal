use std::boxed::Box;
use std::error::Error;
use std::fmt;
use std::sync::mpsc::channel;
use std::thread;
use std::time::{Duration, Instant};

use log;
use redis;

use crate::models::database::Query;
use crate::models::Peripheral;
use crate::plugins::messaging::{Receiver, Transmitter};
use crate::plugins::Plugin;

/// A single task to be executed by the scheduler.
///
/// # Safety
///
/// A `Task` instance is Send because it is created outside of the only thread in which it is
/// used. Implementing Send allows us to move the task into the thread after it is created.
pub struct Task {
    /// A short description of the task that is intended for humans
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
    /// Returns a new instance of a Task.
    ///
    /// # Arguments
    ///
    /// * `description` - A short description of the task that is intended for humans
    /// * `interval` - The minimum amount of time that must elapse between the end of one run of
    /// the task and the start of the next run
    /// * `last_tick` - The time at which the previous run of the task finished
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

unsafe impl Send for Task {}

/// Executes tasks on a Plugin.
///
/// Each Plugin is powered by a single scheduler.
pub struct Scheduler {
    /// The Plugin instance that is managed by this scheduler.
    pub plugin: Plugin,

    /// A connection to the database.
    db: redis::Connection,

    /// A copy of a Peripheral. This is used to update the corresponding entry in the database.
    peripheral: Peripheral,

    /// The scheduler's receiver.
    pub rx: Receiver,

    /// The time between the end of one run of the scheduler and the start of the next.
    sleep: Duration,

    /// A collection of tasks to periodically execute.
    tasks: Vec<Task>,

    /// The scheduler's transmitter.
    pub tx: Transmitter,
}

impl Scheduler {
    /// Returns a new instance of a scheduler.
    ///
    /// # Arguments
    ///
    /// * `plugin` - The Plugin instance that is managed by this Scheduler
    /// * `db` - A connection to the database
    /// * `peripheral` - A copy of a Peripheral. This is used to update the corresponding entry in
    /// the database.
    /// * `sleep` - The time between the end of one run of the scheduler and the start of the next
    pub fn new(
        plugin: Plugin,
        db: redis::Connection,
        peripheral: Peripheral,
        sleep: Duration,
    ) -> Scheduler {
        let tasks: Vec<Task> = Vec::new();
        let (tx, rx) = channel();

        Scheduler {
            plugin,
            db,
            peripheral,
            rx,
            sleep,
            tasks,
            tx,
        }
    }

    /// Adds a new task to the end of the list of Tasks to be executed by the scheduler.
    ///
    /// * `task` - The task to add to the scheduler.
    pub fn push(&mut self, task: Task) {
        self.tasks.push(task);
    }

    /// Starts a Scheduler.
    ///
    /// The Scheduler runs inside an infinite loop. During one iteration of the loop, it executes
    /// each of its Tasks one-by-one from the beginning of the list to the end. If any Tasks were
    /// one, it updates the database entry and then sleeps for a set amount of time.
    ///
    /// This is a function and not a method of a Scheduler instance because the function takes
    /// ownership of the instance.
    ///
    /// # Arguments
    ///
    /// - `scheduler` - A Scheduler instance. This will be consumed by the function and cannot be
    /// used again after this function is called.
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

                log::debug!(
                    "Checking for messages for peripheral: {}",
                    scheduler.peripheral.id()
                );
                // TODO Change this to recv when message passing if fully implemented
                let _ = scheduler.rx.try_recv();

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

/// An error returned by a failed Scheduler thread.
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
