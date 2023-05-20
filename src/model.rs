use std::result;

#[derive(Debug)]
pub struct Error {
  pub message: String,
  pub windows: Option<windows::core::Error>,
}

impl Error {
  pub fn new(message: impl Into<String>) -> Error {
    Error {
      message: message.into(),
      windows: None,
    }
  }

  pub fn windows(message: impl Into<String>, err: windows::core::Error) -> Error {
    Error {
      message: message.into(),
      windows: Some(err),
    }
  }
}

pub type Result<T> = result::Result<T, Error>;
