use std::ptr::null_mut;

use nixrs_sys::{
    nix_c_context, nix_c_context_create, nix_c_context_free, nix_err_code, nix_err_msg,
    NIX_ERR_KEY, NIX_ERR_NIX_ERROR, NIX_ERR_OVERFLOW, NIX_ERR_UNKNOWN, NIX_OK,
};

use crate::utils::{string_from_c, Error, Result};

#[derive(Debug)]
pub struct Context(*mut nix_c_context);

impl Context {
    pub fn new() -> Self {
        let ctx = unsafe { nix_c_context_create() };
        Self(ctx)
    }

    fn check(&self) -> Result<()> {
        let error_code = unsafe { nix_err_code(self.0) };
        if error_code == NIX_OK as i32 {
            return Ok(());
        }
        let msg = unsafe { string_from_c(nix_err_msg(null_mut(), self.0, null_mut()))? };
        let err = match error_code {
            NIX_ERR_UNKNOWN => Error::NixErrorUnknown(msg),
            NIX_ERR_OVERFLOW => Error::NixErrorOverflow(msg),
            NIX_ERR_KEY => Error::NixErrorKey(msg),
            NIX_ERR_NIX_ERROR => Error::NixErrorNixError(msg),
            _ => Error::UnknownError,
        };
        Err(err)
    }

    pub(crate) fn exec<F, O>(&self, mut f: F) -> Result<O>
    where
        F: FnMut(*mut nix_c_context) -> O,
    {
        let output = f(self.0);
        self.check()?;
        Ok(output)
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { nix_c_context_free(self.0) };
    }
}
