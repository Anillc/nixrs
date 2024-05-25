use std::{ffi::CString, ptr::null};
use nixrs_sys::{nix_alloc_value, nix_expr_eval_from_string, nix_state_create, nix_state_free, EvalState};

use crate::{context::Context, store::Store, utils::{NixRSError, Result}, value::Value};

#[derive(Debug)]
pub struct State {
  ctx: Context,
  _store: Store,
  pub(crate) state: *mut EvalState,
}

impl State {
  pub fn new(store: Store) -> Result<State> {
    Self::new_with_paths(store, &[])
  }

  pub fn new_with_paths(store: Store, paths: &[&str]) -> Result<State> {
    let ctx = Context::new();
    let paths: Vec<_> = paths.into_iter()
      .map(|path| CString::new(path.to_string()).map_err(|_| NixRSError::UnknownError))
      .collect::<Result<Vec<CString>>>()?;
    let mut paths_c: Vec<_> = paths.iter().map(|path| path.as_ptr()).collect();
    paths_c.push(null());
    let state = unsafe {
      let state = nix_state_create(ctx.ctx, paths_c.as_mut_ptr(), store.store);
      NixRSError::from_raw(&ctx)?;
      state
    };
    drop(paths_c);
    Ok(State { _store: store, state, ctx })
  }

  pub fn eval(&mut self, expr: &str) -> Result<Value> {
    self.eval_with_path(".", expr)
  }

  pub fn eval_with_path(&mut self, path: &str, expr: &str) -> Result<Value> {
    unsafe {
      let expr = CString::new(expr).map_err(|_| NixRSError::UnknownError)?;
      let path = CString::new(path).map_err(|_| NixRSError::UnknownError)?;
      let value = nix_alloc_value(self.ctx.ctx, self.state); 
      NixRSError::from_raw(&self.ctx)?;
      nix_expr_eval_from_string(self.ctx.ctx, self.state, expr.as_ptr(), path.as_ptr(), value);
      NixRSError::from_raw(&self.ctx)?;
      Value::from_raw(value)
    }
  }
}

impl Drop for State {
  fn drop(&mut self) {
    unsafe { nix_state_free(self.state) };
  }
}
