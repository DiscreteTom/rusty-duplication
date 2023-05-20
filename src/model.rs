use crate::error::Error;
use std::result;

pub type Result<T> = result::Result<T, Error>;
