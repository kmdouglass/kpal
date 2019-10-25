use std::error::Error;
use std::fmt;
use std::fmt::Debug;
use std::sync::mpsc::Receiver as Recv;
use std::sync::mpsc::Sender;

use kpal_plugin::Value;
use log;

use crate::models::database::Query;
use crate::models::{Attribute, Peripheral};
use crate::plugins::driver::{attribute_value, NameError, ValueError};
use crate::plugins::Plugin;

/// Represents a single receiver that is owned by a peripheral.
pub type Receiver = Recv<Message>;

/// Represents a single transmitter for communicating with a peripheral.
pub type Transmitter = Sender<Message>;

/// A message that is passed from a request handler to a peripheral.
pub enum Message {
    GetPeripheralAttribute(usize, Sender<Result<Attribute, PluginError>>),
}

impl Message {
    /// Perform the action requested by a message and transmit the result.
    pub fn handle(&self, peripheral: &mut Peripheral, plugin: &Plugin) {
        match self {
            Message::GetPeripheralAttribute(id, tx) => {
                let mut value = Value::Int(0);
                let result = match attribute_value(plugin, *id, &mut value) {
                    // TODO Use combinators instead
                    Ok(_) => {
                        log::debug!(
                            "Retrieved value {:?} from peripheral {}",
                            value,
                            peripheral.id()
                        );
                        peripheral.set_attribute_from_value(*id, value);
                        let attr = &peripheral.attributes()[*id];

                        Ok(attr.clone())
                    }
                    Err(err) => {
                        log::error!("Message handler error: {:?}", err);
                        let err = PluginError::from(err);

                        Err(err)
                    }
                };

                log_and_send(tx.clone(), result, peripheral.id());
            }
        };
    }
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
            "Successfully sent response back to request from peripheral {}",
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
    /// The HTTP status code that should be returned to the client.
    pub http_status_code: u16,
}

impl Error for PluginError {}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "An error occured when trying to communicate with the plugin"
        )
    }
}

impl From<ValueError> for PluginError {
    fn from(error: ValueError) -> Self {
        match error {
            ValueError::DoesNotExist => PluginError {
                http_status_code: 404,
            },
            ValueError::Failure => PluginError {
                http_status_code: 500,
            },
        }
    }
}

impl From<NameError> for PluginError {
    fn from(error: NameError) -> Self {
        match error {
            NameError::DoesNotExist => PluginError {
                http_status_code: 404,
            },
            NameError::Failure => PluginError {
                http_status_code: 500,
            },
        }
    }
}
