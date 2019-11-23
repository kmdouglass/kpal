use std::error::Error;
use std::fmt;
use std::sync::mpsc::channel;
use std::thread;

use log;

use super::messaging::{Receiver, Transmitter};
use super::Plugin;

use crate::models::Model;
use crate::models::Peripheral;

/// Executes tasks on a Plugin.
///
/// Each Plugin is powered by a single scheduler.
pub struct Scheduler {
    /// The Plugin instance that is managed by this scheduler.
    pub plugin: Plugin,

    /// A copy of a Peripheral model.
    peripheral: Peripheral,

    /// The scheduler's receiver.
    pub rx: Receiver,

    /// The scheduler's transmitter.
    pub tx: Transmitter,
}

impl Scheduler {
    /// Returns a new instance of a scheduler.
    ///
    /// # Arguments
    ///
    /// * `plugin` - The Plugin instance that is managed by this Scheduler
    /// * `peripheral` - A copy of a Peripheral. This is used to update the corresponding model
    pub fn new(plugin: Plugin, peripheral: Peripheral) -> Scheduler {
        let (tx, rx) = channel();

        Scheduler {
            plugin,
            peripheral,
            rx,
            tx,
        }
    }

    /// Starts a Scheduler.
    ///
    /// The Scheduler runs inside an infinite loop. During one iteration of the loop, it checks for
    /// a new message in its message queue. If found, it processes the message (possibly by
    /// communicating with the peripheral through the plugin interface) and returns the via the
    /// return transmitter that was passed alongside the message.
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

            loop {
                log::debug!(
                    "Checking for messages for peripheral: {}",
                    scheduler.peripheral.id()
                );
                let msg = scheduler.rx.recv().map_err(|_| SchedulerRuntimeError {})?;
                msg.handle(&mut scheduler.peripheral, &scheduler.plugin);
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
