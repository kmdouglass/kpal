//! Messages and handlers for communications between peripheral threads and web server requests.

use std::{error::Error, fmt, fmt::Debug, sync::mpsc::Receiver as Recv, sync::mpsc::Sender};

use kpal_plugin::Value;
use log;

use super::executor::{NameError, SetValueError, ValueError};
use super::Executor;

use crate::models::Model;
use crate::models::{Attribute, Peripheral};

/// Represents a single receiver that is owned by a peripheral.
pub type Receiver = Recv<Message>;

/// Represents a single transmitter for communicating with a peripheral.
pub type Transmitter = Sender<Message>;

/// A message that is passed from a request handler to a peripheral.
pub enum Message {
    GetPeripheral(Sender<Result<Peripheral, PluginError>>),
    GetPeripheralAttribute(usize, Sender<Result<Attribute, PluginError>>),
    GetPeripheralAttributes(Sender<Result<Vec<Attribute>, PluginError>>),
    PatchPeripheralAttribute(usize, Value, Sender<Result<Attribute, PluginError>>),
}

impl Message {
    /// Perform the action requested by a message and transmit the result.
    ///
    /// # Arguments
    ///
    /// * `peripheral` - The peripheral to communicate with
    /// * `plugin` - The plugin that communicates with the peripheral
    pub fn handle(&self, ex: &mut Executor) {
        match self {
            Message::GetPeripheral(tx) => {
                log_and_send(tx.clone(), Ok(ex.peripheral.clone()), ex.peripheral.id())
            }

            Message::GetPeripheralAttribute(id, tx) => {
                let result = attribute_value_wrapper(ex, *id);

                log_and_send(tx.clone(), result, ex.peripheral.id());
            }

            Message::GetPeripheralAttributes(tx) => {
                let ids = {
                    let attrs = ex.peripheral.attributes();
                    let mut ids = Vec::new();
                    for attr in attrs {
                        ids.push(attr.id());
                    }
                    ids
                };

                let mut attrs = Vec::new();
                for id in &ids {
                    let result = attribute_value_wrapper(ex, *id);
                    attrs.push(result);
                }

                log_and_send(tx.clone(), attrs.into_iter().collect(), ex.peripheral.id());
            }

            Message::PatchPeripheralAttribute(id, value, tx) => {
                let result = set_attribute_value_wrapper(ex, *id, value);

                log_and_send(tx.clone(), result, ex.peripheral.id());
            }
        };
    }
}

/// Wraps the executor's attribute_value function.
///
/// This function is provided for ergonomics. It keeps the `handle()` function DRY and easier to
/// read.
///
/// # Arguments
///
/// * `executor` - A reference to the current executor instance
/// * `id` - The id of the attribute to fetch
fn attribute_value_wrapper(executor: &mut Executor, id: usize) -> Result<Attribute, PluginError> {
    let mut value = Value::Int(0);
    executor
        .attribute_value(id, &mut value)
        .map(|_| {
            log::debug!(
                "Retrieved value {:?} from peripheral {}",
                value,
                executor.peripheral.id(),
            );
            executor.peripheral.set_attribute_from_value(id, value);
            let attr = &executor.peripheral.attributes()[id];
            attr.clone()
        })
        .map_err(|e| {
            log::error!("Message handler error: {:?}", e);
            PluginError::from(e)
        })
}

/// Wraps the driver's set_attribute_value function.
///
/// This function is provided for ergonomics. It keeps the `handle()` function DRY and easier to
/// read.
///
/// # Arguments
///
/// * `executor` - A reference to the current executor instance
/// * `id` - The id of the attribute to fetch
/// * `value` - The value to set on the attribute
fn set_attribute_value_wrapper(
    executor: &mut Executor,
    id: usize,
    value: &Value,
) -> Result<Attribute, PluginError> {
    executor
        .set_attribute_value(id, value)
        .map(|_| {
            log::debug!(
                "Set value {:?} on peripheral {}",
                value,
                executor.peripheral.id(),
            );
            executor
                .peripheral
                .set_attribute_from_value(id, value.clone());
            let attr = &executor.peripheral.attributes()[id];
            attr.clone()
        })
        .map_err(|e| {
            log::error!("Message handler error: {:?}", e);
            PluginError::from(e)
        })
}

/// Sends a response back to the requesting thread.
///
/// # Arguments
///
/// * `tx` - The sender used to return a response.
/// * `result` - The result object to return
/// * `peripheral_id` The ID of the peripheral from which the response originates
fn log_and_send<T: Debug>(
    tx: Sender<Result<T, PluginError>>,
    result: Result<T, PluginError>,
    peripheral_id: usize,
) {
    if let Err(err) = tx.send(result) {
        log::error!(
            "Failed to return response from peripheral: {}. Reason: {}",
            peripheral_id,
            err
        );
    } else {
        log::debug!(
            "Successfully sent response back to request of peripheral: {}",
            peripheral_id
        );
    };
}

/// Contains information for clients about errors that occur while communicating with a plugin.
///
/// This error type is intended for the exclusive use by server request handlers. After an
/// operation is performed on a plugin, the result may be an error of one of many different
/// types. These errors should be converted into a PluginError. A PluginError contains the
/// information that is necessary for the server's request handler to report information back to
/// the client about why the requested operation failed.
#[derive(Debug)]
pub struct PluginError {
    /// The body of the HTTP response to return to the client.
    pub body: String,

    /// The HTTP status code that should be returned to the client.
    pub http_status_code: u16,
}

impl Error for PluginError {}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "PluginError: {:?}", self)
    }
}

impl From<NameError> for PluginError {
    fn from(error: NameError) -> Self {
        match error {
            NameError::DoesNotExist(msg) => PluginError {
                body: msg,
                http_status_code: 404,
            },
            NameError::Failure(msg) => PluginError {
                body: msg,
                http_status_code: 500,
            },
        }
    }
}

impl From<ValueError> for PluginError {
    fn from(error: ValueError) -> Self {
        match error {
            ValueError::DoesNotExist(msg) => PluginError {
                body: msg,
                http_status_code: 404,
            },
            ValueError::Failure(msg) => PluginError {
                body: msg,
                http_status_code: 500,
            },
        }
    }
}

impl From<SetValueError> for PluginError {
    fn from(error: SetValueError) -> Self {
        match error {
            SetValueError::DoesNotExist(msg) => PluginError {
                body: msg,
                http_status_code: 404,
            },
            SetValueError::Failure(msg) => PluginError {
                body: msg,
                http_status_code: 500,
            },
        }
    }
}
