use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error, Clone)]
pub enum Error {
  #[error("no output available")]
  NoOutput,

  #[error("invalid buffer length")]
  InvalidBufferLength,

  /// A Windows error.
  #[error("{api}: {err}")]
  Windows {
    api: &'static str,
    err: windows::core::Error,
  },
}

impl Error {
  /// Create a new Windows error.
  #[inline]
  pub(crate) const fn windows(api: &'static str, err: windows::core::Error) -> Self {
    Self::Windows { api, err }
  }

  /// Create a new Windows error from `GetLastError`.
  #[inline]
  pub(crate) fn from_win32(api: &'static str) -> Self {
    Self::windows(api, windows::core::Error::from_win32())
  }
}

#[cfg(test)]
mod tests {
  use super::Error;

  #[test]
  fn format_error() {
    assert_eq!(format!("{}", Error::NoOutput), "no output available");

    assert_eq!(
      format!("{}", Error::InvalidBufferLength),
      "invalid buffer length"
    );

    assert_eq!(
      format!(
        "{}",
        Error::Windows {
          api: "api",
          err: windows::core::Error::empty()
        }
      ),
      "api: The operation completed successfully. (0x00000000)"
    );
  }
}
