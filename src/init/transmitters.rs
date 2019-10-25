use std::collections::HashMap;
use std::sync::{Mutex, RwLock};

use crate::plugins::messaging::Transmitter;

/// A set of distinct transmitters.
pub type Transmitters = RwLock<HashMap<usize, Mutex<Transmitter>>>;

/// Returns an empty collection of thread transmitters.
pub fn init() -> Transmitters {
    let map: HashMap<usize, Mutex<Transmitter>> = HashMap::new();

    RwLock::new(map)
}
