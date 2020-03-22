use std::collections::HashMap;
use std::sync::Mutex;

use crate::plugins::Transmitter;

/// A set of distinct transmitters for sending messages into executor threads.
pub type Transmitters = HashMap<usize, Mutex<Transmitter>>;

/// Returns an empty collection of thread transmitters.
pub fn init() -> Transmitters {
    HashMap::new()
}
