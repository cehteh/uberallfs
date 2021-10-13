pub use std::io;

#[allow(unused_imports)]
pub use uberall::log::{debug, error, info, trace, warn};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub use crate::errors::ObjectStoreError;
