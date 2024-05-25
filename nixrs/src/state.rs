use std::{collections::HashMap, ffi::CString, ptr::null};
use nixrs_sys::{nix_expr_eval_from_string, nix_state_create, nix_state_free, nix_store_realise, EvalState};

use crate::{context::Context, path::StorePath, store::Store, utils::{build_cb, NixRSError, Result}, value::{Value, ValueType}};

#[derive(Debug)]
pub struct State {
  ctx: Context,
  pub(crate) store: Store,
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
      ctx.check()?;
      state
    };
    drop(paths_c);
    Ok(State { store, state, ctx })
  }

  pub fn eval(&mut self, expr: &str) -> Result<Value> {
    self.eval_with_path(".", expr)
  }

  pub fn eval_with_path(&mut self, path: &str, expr: &str) -> Result<Value> {
    unsafe {
      let expr = CString::new(expr).map_err(|_| NixRSError::UnknownError)?;
      let path = CString::new(path).map_err(|_| NixRSError::UnknownError)?;
      let value = Value::new(&self)?;
      nix_expr_eval_from_string(self.ctx.ctx, self.state, expr.as_ptr(), path.as_ptr(), value.value);
      self.ctx.check()?;
      Ok(value)
    }
  }

  pub fn store_path(&mut self, path: &str) -> Result<StorePath> {
    StorePath::new(&self.store, path)
  }

  pub fn build(&mut self, value: &Value) -> Result<HashMap<String, String>> {
    let ValueType::Attrs = value.get_type()? else { return Err(NixRSError::NotDerivation); };
    let store_path = value.attrs_get(&self, "drvPath")?;
    let ValueType::String = store_path.get_type()? else { return Err(NixRSError::NotDerivation); };
    let store_path = StorePath::new(&self.store, store_path.string()?.as_str())?;
    let map = unsafe {
      let mut map = HashMap::<String, String>::new();
      nix_store_realise(
        self.ctx.ctx, self.store.store, store_path.store_path,
        &mut map as *mut _ as *mut _,
        Some(build_cb),
      );
      self.ctx.check()?;
      map
    };
    Ok(map)
  }
}

impl Drop for State {
  fn drop(&mut self) {
    unsafe { nix_state_free(self.state) };
  }
}
