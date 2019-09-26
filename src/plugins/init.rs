//! Initialization routines for a Plugin.
use std::boxed::Box;
use std::time::Instant;

use log;

use kpal_plugin::Value;

use crate::constants::*;
use crate::models::database::Query;
use crate::models::{Attribute, Peripheral};
use crate::plugins::driver::{attribute_name, attribute_value, NameResult, ValueResult};
use crate::plugins::scheduler::{Scheduler, Task};
use crate::plugins::Plugin;

/// Gets all attribute values and names from a Plugin and updates the corresponding Peripheral.
///
/// # Arguments
///
/// * `peripheral` - The Peripheral instance to update
/// * `plugin` - The plugin whose attributes will be fetched
pub fn attributes(peripheral: &mut Peripheral, plugin: &Plugin) {
    log::info!("Getting attributes for peripheral {}", peripheral.id());

    let mut value = Value::Int(0);
    let mut name = [0u8; ATTRIBUTE_NAME_BUFFER_LENGTH];
    let mut index = 0;
    let mut attr: Vec<Attribute> = Vec::new();

    loop {
        match attribute_value(plugin, index, &mut value) {
            ValueResult::Success => (),
            ValueResult::DoesNotExist => break,
            ValueResult::Failure => {
                index += 1;
                continue;
            }
        };

        let name = match attribute_name(plugin, index, &mut name) {
            NameResult::Success(name) => name,
            NameResult::DoesNotExist => break,
            NameResult::Failure => {
                index += 1;
                continue;
            }
        };

        attr.push(Attribute::from(value.clone(), index, name));

        index += 1;
    }

    peripheral.set_attributes(attr);
}

/// Creates the tasks for getting attribute values from a plugin.
///
/// The callback field is a curried function that contains information specific to an attribute.
///
/// # Arguments
///
/// * `peripheral` - The Peripheral instance from which attributes will be obtained
/// * `scheduler` - A Scheduler instance to populate with the new tasks
pub fn tasks(peripheral: &Peripheral, scheduler: &mut Scheduler) {
    let start_now = Instant::now() - TASK_INTERVAL_DURATION;
    for attr in peripheral.attributes() {
        scheduler.push(Task::new(
            String::from(format!(
                "Get attribute {} from peripheral {}",
                attr.id(),
                peripheral.id()
            )),
            TASK_INTERVAL_DURATION,
            start_now,
            Box::new(attribute_value_callback(attr.id())),
        ));
    }
}

/// Returns a function used by the scheduler to get the value of an attribute from the peripheral.
///
/// # Arguments
///
/// * `id` - The numeric ID of the attribute. This will be embedded into the callback function that
/// is returned and will not need to be explicitly passed when calling the callback.
///
/// # Returns
///
/// A function that will be called when the corresponding Task is executed by a Scheduler.
fn attribute_value_callback(id: usize) -> impl Fn(&mut Peripheral, &Plugin) {
    move |peripheral: &mut Peripheral, plugin: &Plugin| {
        let mut value = Value::Int(0);
        match attribute_value(plugin, id, &mut value) {
            ValueResult::Success => peripheral.set_attribute_from_value(id, value),
            _ => (),
        };
    }
}
