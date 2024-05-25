use std::{ffi::CStr, ptr::null_mut};

use nixrs_sys::{nix_err_code, nix_err_msg, NIX_ERR_KEY, NIX_ERR_NIX_ERROR, NIX_ERR_OVERFLOW, NIX_ERR_UNKNOWN, NIX_OK};
use thiserror::Error;

use crate::context::Context;

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

impl NixRSError {
  pub(crate) unsafe fn from_raw(ctx: &Context) -> Result<()> {
    let err = nix_err_code(ctx.ctx);
    if err == NIX_OK as i32 { return Ok(()); }
    let msg = match string_from_c(nix_err_msg(null_mut(), ctx.ctx, null_mut())) {
      Ok(msg) => msg,
      Err(err) => return Err(err),
    };
    match err {
      NIX_ERR_UNKNOWN => Err(NixRSError::NixErrorUnknown(msg)),
      NIX_ERR_OVERFLOW => Err(NixRSError::NixErrorOverflow(msg)),
      NIX_ERR_KEY => Err(NixRSError::NixErrorKey(msg)),
      NIX_ERR_NIX_ERROR => Err(NixRSError::NixErrorNixError(msg)),
      _ => Err(NixRSError::UnknownError),
    }
  }
}

pub(crate) unsafe fn string_from_c(ptr: *const libc::c_char) -> Result<String> {
  CStr::from_ptr(ptr).to_str().map(|str| str.to_string()).map_err(|_| NixRSError::UnknownError)
}
