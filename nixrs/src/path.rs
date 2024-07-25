use nixrs_sys::{nix_store_parse_path, nix_store_path_free, nix_store_path_name};
use std::ffi::CString;

use crate::{
    context::Context,
    store::Store,
    utils::{get_string_cb, Error, Result},
};

#[derive(Debug)]
pub struct StorePath {
    pub(crate) store_path: *mut nixrs_sys::StorePath,
}

impl StorePath {
    pub(crate) fn new(store: &Store, path: &str) -> Result<Self> {
        let path = CString::new(path).map_err(|_| Error::UnknownError)?;
        let store_path = Context::new()
            .exec(|ctx| unsafe { nix_store_parse_path(ctx, store.store, path.as_ptr()) })?;
        Ok(Self { store_path })
    }

    pub fn name(&self) -> Result<String> {
        let mut vec: Vec<u8> = Vec::new();
        unsafe {
            nix_store_path_name(
                self.store_path,
                Some(get_string_cb),
                &mut vec as *mut _ as *mut _,
            );
        }
        Ok(String::from_utf8(vec).map_err(|_| Error::UnknownError)?)
    }
}

impl Drop for StorePath {
    fn drop(&mut self) {
        unsafe { nix_store_path_free(self.store_path) };
    }
}
