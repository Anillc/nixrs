use std::ffi::CStr;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, NixRSError>;

#[derive(Debug, Error)]
pub enum NixRSError {
  #[error("NIX_ERROR_UNKNOWN: {0}")]
  NixErrorUnknown(String),
  #[error("NIX_ERROR_OVERFLOW: {0}")]
  NixErrorOverflow(String),
  #[error("NIX_ERROR_KEY: {0}")]
  NixErrorKey(String),
  #[error("NIX_ERROR_NIX_ERROR: {0}")]
  NixErrorNixError(String),
  #[error("unknown error")]
  UnknownError,
}

pub(crate) unsafe fn string_from_c(ptr: *const libc::c_char) -> Result<String> {
  CStr::from_ptr(ptr).to_str().map(|str| str.to_string()).map_err(|_| NixRSError::UnknownError)
}
