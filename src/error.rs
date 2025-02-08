use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
  #[error("invalid buffer length")]
  InvalidBufferLength,
  #[error("no output available")]
  NoOutput,
  /// A Windows error.
  #[error("{api}: {err}")]
  Windows {
    api: &'static str,
    err: windows::core::Error,
  },
}

impl Error {
  pub(crate) fn windows(api: &'static str, err: windows::core::Error) -> Error {
    Error::Windows { api, err }
  }

  pub(crate) fn from_win32(api: &'static str) -> Error {
    Error::Windows {
      api,
      err: windows::core::Error::from_win32(),
    }
  }
}
