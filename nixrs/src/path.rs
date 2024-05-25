use std::ffi::CString;
use nixrs_sys::{nix_store_parse_path, nix_store_path_free, nix_store_path_name};

use crate::{context::Context, store::Store, utils::{get_string_cb, NixRSError, Result}};

#[derive(Debug)]
pub struct StorePath {
  pub(crate) store_path: *mut nixrs_sys::StorePath,
}

impl StorePath {
  pub(crate) fn new(store: &Store, path: &str) -> Result<StorePath> {
    let ctx = Context::new();
    let store_path = unsafe {
      let path = CString::new(path).map_err(|_| NixRSError::UnknownError)?;
      let store_path = nix_store_parse_path(ctx.ctx, store.store, path.as_ptr());
      ctx.check()?;
      store_path
    };
    Ok(StorePath { store_path })
  }

  pub fn name(&self) -> Result<String> {
    unsafe {
      let mut vec: Vec<u8> = Vec::new();
      nix_store_path_name(self.store_path, Some(get_string_cb), &mut vec as *mut _ as *mut _);
      Ok(String::from_utf8(vec).map_err(|_| NixRSError::UnknownError)?)
    }
  }
}

impl Drop for StorePath {
  fn drop(&mut self) {
    unsafe { nix_store_path_free(self.store_path) };
  }
}
