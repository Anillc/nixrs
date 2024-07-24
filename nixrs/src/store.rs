use std::{ffi::CString, ptr::null_mut};

use nixrs_sys::{nix_store_free, nix_store_open};

use crate::{
    context::Context,
    utils::{Error, Result},
};

#[derive(Debug)]
pub struct Store {
    pub(crate) store: *mut nixrs_sys::Store,
}

impl Store {
    pub fn new(uri: &str) -> Result<Self> {
        let uri = CString::new(uri).map_err(|_| Error::UnknownError)?;
        let ctx = Context::new();
        let store = ctx.exec(|ctx| unsafe { nix_store_open(ctx, uri.as_ptr(), null_mut()) })?;
        drop(uri);
        Ok(Self { store })
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        unsafe { nix_store_free(self.store) };
    }
}
