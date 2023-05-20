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

impl std::fmt::Display for Error {
  fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self.windows {
      Some(ref err) => std::write!(fmt, "{} ({})", self.message, err),
      None => std::write!(fmt, "{}", self.message),
    }
  }
}

impl std::error::Error for Error {}
