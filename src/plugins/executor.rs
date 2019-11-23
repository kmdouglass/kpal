use std::error::Error;
use std::fmt;
use std::sync::mpsc::channel;
use std::thread;

use log;

use super::messaging::{Receiver, Transmitter};
use super::Plugin;

use crate::init::libraries::TSLibrary;
use crate::models::Model;
use crate::models::Peripheral;

/// Executes tasks on a Plugin in response to messages.
///
/// Each Plugin is powered by a single executor.
pub struct Executor {
    /// The Plugin instance that is managed by this executor.
    pub plugin: Plugin,

    /// A copy of a Peripheral model.
    pub peripheral: Peripheral,

    /// The executor's receiver.
    pub rx: Receiver,

    /// The executor's transmitter.
    pub tx: Transmitter,

    /// The library of the plugin controlled by the executor.
    pub lib: TSLibrary,
}

impl Executor {
    /// Returns a new instance of an executor.
    ///
    /// # Arguments
    ///
    /// * `plugin` - The Plugin instance that is managed by this Executor
    /// * `peripheral` - A copy of a Peripheral. This is used to update the corresponding model
    pub fn new(plugin: Plugin, peripheral: Peripheral, lib: TSLibrary) -> Executor {
        let (tx, rx) = channel();

        Executor {
            plugin,
            peripheral,
            rx,
            tx,
            lib,
        }
    }

    /// Starts a Executor.
    ///
    /// The Executor runs inside an infinite loop. During one iteration of the loop, it checks for
    /// a new message in its message queue. If found, it processes the message (possibly by
    /// communicating with the peripheral through the plugin interface) and returns the via the
    /// return transmitter that was passed alongside the message.
    ///
    /// This is a function and not a method of a Executor instance because the function takes
    /// ownership of the instance.
    ///
    /// # Arguments
    ///
    /// - `executor` - A Executor instance. This will be consumed by the function and cannot be
    /// used again after this function is called.
    pub fn run(mut self) {
        thread::spawn(move || -> Result<(), ExecutorRuntimeError> {
            log::info!("Spawning new thread for plugin: {:?}", self.plugin);

            loop {
                log::debug!(
                    "Checking for messages for peripheral: {}",
                    self.peripheral.id()
                );
                let msg = self.rx.recv().map_err(|_| ExecutorRuntimeError {})?;
                msg.handle(&mut self);
            }
        });
    }
}

/// An error returned by a failed Executor thread.
#[derive(Debug)]
pub struct ExecutorRuntimeError {}

impl Error for ExecutorRuntimeError {
    fn description(&self) -> &str {
        "The executor thread failed"
    }
}

impl fmt::Display for ExecutorRuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "The executor thread failed")
    }
}
