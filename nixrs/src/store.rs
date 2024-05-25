use std::ptr::null_mut;

use std::ffi::CString;
use nixrs_sys::{nix_store_free, nix_store_open};

use crate::{context::Context, utils::{NixRSError, Result}};

#[derive(Debug)]
pub struct Store {
  pub(crate) store: *mut nixrs_sys::Store,
}

impl Store {
  pub fn new(uri: &str) -> Result<Store> {
    let ctx = Context::new();
    let store = unsafe {
      let uri = CString::new(uri).map_err(|_| NixRSError::UnknownError)?;
      let store = nix_store_open(ctx.ctx, uri.as_ptr(), null_mut());
      NixRSError::from_raw(&ctx)?;
      drop(uri);
      store
    };
    Ok(Store { store })
  }
}

impl Drop for Store {
  fn drop(&mut self) {
    unsafe { nix_store_free(self.store) };
  }
}
