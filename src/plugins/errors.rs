//! Error types for the plugins module.

use std::{
    error::Error,
    fmt,
    fmt::Debug,
    sync::{MutexGuard, PoisonError, RwLockWriteGuard},
};

use crate::{
    init::Transmitters,
    models::{Library, ModelError},
};

/// Contains information for clients about errors that occur while communicating with a plugin.
#[derive(Debug)]
pub enum PluginError {
    AdvancePhaseError(i32),
    AttributeCountError,
    AttributeIDsError,
    AttributeDoesNotExist(String),
    AttributeFailure(String),
    AttributeNotSettable(String),
    ChannelReceiveError(std::sync::mpsc::RecvError),
    GetLibraryError(String),
    GetTransmittersError(String),
    MessageNullPointerError,
    ModelFailure(ModelError),
    NewPluginError,
    PluginInitError(String),
    SetAttributesFailure(String),
    SetAttributesUserInputError(String),
    SymbolError(std::io::Error),
    Utf8Error(std::str::Utf8Error),
}

impl Error for PluginError {}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use PluginError::*;
        match self {
            AdvancePhaseError(phase) => write!(
                f,
                "could not advance the lifetime phase of the plugin from phase {}",
                phase
            ),
            AttributeCountError => write!(f, "could not determine the number of plugin attributes"),
            AttributeIDsError => write!(f, "could not determine the attribute IDs"),
            AttributeDoesNotExist(e) => write!(f, "attribute does not exist\nCaused by: {}", e),
            AttributeFailure(e) => {
                write!(f, "could not get or set attribute value\nCaused by: {}", e)
            }
            AttributeNotSettable(e) => write!(f, "attribute is not settable\nCaused by: {}", e),
            ChannelReceiveError(e) => {
                write!(f, "could not read message from the plugin's channel: {}", e)
            }
            GetLibraryError(e) => write!(f, "could not get library for plugin\nCaused by: {}", e),
            GetTransmittersError(e) => {
                write!(f, "could not get transmitter for plugin\nCaused by: {}", e)
            }
            MessageNullPointerError => write!(
                f,
                "the plugin returned a null pointer instead of an error message"
            ),
            ModelFailure(e) => write!(
                f,
                "encountered error in the KPAL object model\nCaused by: {}",
                e
            ),
            NewPluginError => write!(f, "could not create new plugin instance"),
            PluginInitError(e) => write!(f, "could not initialize plugin\nCaused by: {}", e),
            SetAttributesFailure(e) => write!(
                f,
                "could not merge user-specified attributes into defaults\nCaused by: {}",
                e
            ),
            SetAttributesUserInputError(e) => write!(
                f,
                "user-defined pre-init value(s) is invalid\nCaused by: {}",
                e
            ),
            SymbolError(e) => write!(
                f,
                "could not get symbol from shared library\nCaused by: {}",
                e
            ),
            Utf8Error(e) => write!(f, "could not parse attribute name\nCaused by: {}", e),
        }
    }
}

impl From<ModelError> for PluginError {
    fn from(error: ModelError) -> Self {
        PluginError::ModelFailure(error)
    }
}

impl From<std::io::Error> for PluginError {
    fn from(error: std::io::Error) -> Self {
        PluginError::SymbolError(error)
    }
}

impl<'a> From<PoisonError<MutexGuard<'a, Library>>> for PluginError {
    fn from(_: PoisonError<MutexGuard<Library>>) -> Self {
        PluginError::GetLibraryError("The mutex on the library is poisoned".to_owned())
    }
}

impl<'a> From<PoisonError<RwLockWriteGuard<'a, Transmitters>>> for PluginError {
    fn from(_: PoisonError<RwLockWriteGuard<Transmitters>>) -> Self {
        PluginError::GetTransmittersError(
            "The RwLock on the transmitters collection is poisoned".to_owned(),
        )
    }
}

impl From<std::sync::mpsc::RecvError> for PluginError {
    fn from(error: std::sync::mpsc::RecvError) -> Self {
        PluginError::ChannelReceiveError(error)
    }
}

impl From<std::str::Utf8Error> for PluginError {
    fn from(error: std::str::Utf8Error) -> Self {
        PluginError::Utf8Error(error)
    }
}
