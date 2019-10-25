use std::collections::HashMap;
use std::sync::Mutex;

use crate::plugins::messaging::Transmitter;

/// A set of distinct transmitters.
pub type Transmitters = HashMap<usize, Mutex<Transmitter>>;

/// Returns an empty collection of thread transmitters.
pub fn init() -> Transmitters {
    HashMap::new()
}
