use crate::error::Error;
use std::result;

pub type Result<T> = result::Result<T, Error>;

pub struct MouseUpdateStatus {
  pub position_updated: bool,
  pub shape_updated: bool,
}
