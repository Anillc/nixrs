use std::{ffi::CString, ptr::null};
use nixrs_sys::{nix_state_create, nix_state_free, EvalState};

use crate::{context::Context, store::Store, utils::{NixRSError, Result}};

#[derive(Debug)]
pub struct State {
  state: *mut EvalState,
}

impl State {
  pub fn new(store: &Store) -> Result<State> {
    Self::new_paths(store, &[])
  }

  pub fn new_paths(store: &Store, paths: &[&str]) -> Result<State> {
    let ctx = Context::new();
    let paths: Vec<_> = paths.into_iter()
      .map(|path| CString::new(path.to_string()).map_err(|_| NixRSError::UnknownError))
      .collect::<Result<Vec<CString>>>()?;
    let mut paths_c: Vec<_> = paths.iter().map(|path| path.as_ptr()).collect();
    paths_c.push(null());
    let state = unsafe {
      let state = nix_state_create(ctx.ctx, paths_c.as_mut_slice().as_mut_ptr(), store.store);
      NixRSError::from_raw(&ctx)?;
      state
    };
    Ok(State { state })
  }
}

impl Drop for State {
  fn drop(&mut self) {
    unsafe { nix_state_free(self.state) };
  }
}
