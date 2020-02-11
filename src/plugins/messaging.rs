//! Messages and handlers for communications between peripheral threads and web server requests.

use std::{fmt::Debug, sync::mpsc::Receiver as Recv, sync::mpsc::Sender};

use kpal_plugin::Val as PluginValue;
use log;

use super::{Executor, PluginError};

use crate::models::{Attribute, Model, Peripheral, Value};

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
    /// * `ex` - A reference to the executor that controls the plugin
    /// * `periph` - A reference to the peripheral model that maintains the peripheral state
    pub fn handle(&self, ex: &mut Executor, periph: &mut Peripheral) {
        match self {
            Message::GetPeripheral(tx) => log_and_send(tx.clone(), Ok(periph.clone()), periph.id()),

            Message::GetPeripheralAttribute(id, tx) => {
                let result = attribute_value_wrapper(ex, periph, *id);

                log_and_send(tx.clone(), result, periph.id());
            }

            Message::GetPeripheralAttributes(tx) => {
                let ids = {
                    let mut ids = Vec::new();
                    for id in periph.attributes().keys() {
                        ids.push(*id);
                    }
                    ids
                };

                let mut attrs = Vec::new();
                for id in &ids {
                    let result = attribute_value_wrapper(ex, periph, *id);
                    attrs.push(result);
                }

                log_and_send(tx.clone(), attrs.into_iter().collect(), periph.id());
            }

            Message::PatchPeripheralAttribute(id, value, tx) => {
                let value: PluginValue = value.as_val();
                let result = set_attribute_value_wrapper(ex, periph, *id, value);

                log_and_send(tx.clone(), result, periph.id());
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
fn attribute_value_wrapper(
    ex: &mut Executor,
    periph: &mut Peripheral,
    id: usize,
) -> Result<Attribute, PluginError> {
    let mut value = PluginValue::Int(0);
    ex.attribute_value(id, &mut value)
        .map(|_| {
            log::debug!(
                "Retrieved value {:?} from peripheral {}",
                value,
                periph.id(),
            );
        })
        .map_err(|e| {
            log::error!("Message handler error: {:?}", e);
            PluginError::from(e)
        })?;

    periph.set_attribute_from_value(id, value)?;
    let attr = &periph.attributes()[&id];
    Ok(attr.clone())
}

/// Wraps the driver's set_attribute_value function.
///
/// This function is provided for ergonomics. It keeps the `handle()` function DRY and easier to
/// read.
///
/// # Arguments
///
/// * `ex` - A reference to the current executor instance
/// * `periph` - A reference to the perhipheral model that maintains the peripheral's state
/// * `id` - The id of the attribute to fetch
/// * `value` - The value to set on the attribute
fn set_attribute_value_wrapper(
    ex: &mut Executor,
    periph: &mut Peripheral,
    id: usize,
    value: PluginValue,
) -> Result<Attribute, PluginError> {
    ex.set_attribute_value(id, &value)
        .map(|_| {
            log::debug!("Set value {:?} on peripheral {}", value, periph.id(),);
        })
        .map_err(|e| {
            log::error!("Message handler error: {:?}", e);
            PluginError::from(e)
        })?;

    periph.set_attribute_from_value(id, value)?;
    let attr = &periph.attributes()[&id];
    Ok(attr.clone())
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
