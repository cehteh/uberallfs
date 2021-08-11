#[allow(unused_imports)]
pub use log::{debug, error, info, trace, warn};

pub use std::io;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub use crate::errors::ObjectStoreError;
