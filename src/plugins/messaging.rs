use std::sync::mpsc::Receiver as Recv;
use std::sync::mpsc::Sender;

/// Represents a single receiver that is owned by a peripheral.
pub type Receiver = Recv<Message>;

/// Represents a single transmitter for communicating with a peripheral.
pub type Transmitter = Sender<Message>;

/// A message that is passed from a request handler to a peripheral.
pub enum Message {
    Dummy,
}
