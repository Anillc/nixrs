use std::ptr::null_mut;

use nixrs_sys::{nix_c_context, nix_c_context_create, nix_c_context_free, nix_err_code, nix_err_msg, NIX_ERR_KEY, NIX_ERR_NIX_ERROR, NIX_ERR_OVERFLOW, NIX_ERR_UNKNOWN, NIX_OK};

use crate::utils::{string_from_c, NixRSError, Result};

#[derive(Debug)]
pub struct Context {
  pub(crate) ctx: *mut nix_c_context,
}

impl Context {
  pub fn new() -> Context {
    let ctx = unsafe { nix_c_context_create() };
    Context { ctx }
  }

  pub unsafe fn check(&self) -> Result<()> {
    let err = nix_err_code(self.ctx);
    if err == NIX_OK as i32 { return Ok(()); }
    let msg = match string_from_c(nix_err_msg(null_mut(), self.ctx, null_mut())) {
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

impl Drop for Context {
  fn drop(&mut self) {
    unsafe { nix_c_context_free(self.ctx) };
  }
}
