use std::{collections::HashMap, ffi::CString, ptr::null};

use nixrs_sys::{
    nix_expr_eval_from_string, nix_state_create, nix_state_free, nix_store_realise, EvalState,
};

use crate::{
    context::Context,
    path::StorePath,
    store::Store,
    utils::{build_cb, Error, Result},
    value::{Value, ValueType},
};

#[derive(Debug)]
pub struct State {
    ctx: Context,
    pub(crate) store: Store,
    pub(crate) state: *mut EvalState,
}

impl State {
    pub fn new(store: Store) -> Result<Self> {
        Self::new_with_paths(store, &[])
    }

    pub fn new_with_paths(store: Store, paths: &[&str]) -> Result<Self> {
        let ctx = Context::new();
        let paths: Vec<_> = paths
            .into_iter()
            .map(|path| CString::new(path.to_string()).map_err(|_| Error::UnknownError))
            .collect::<Result<Vec<CString>>>()?;
        let mut paths_c: Vec<_> = paths.iter().map(|path| path.as_ptr()).collect();
        paths_c.push(null());
        let state =
            ctx.exec(|ctx| unsafe { nix_state_create(ctx, paths_c.as_mut_ptr(), store.store) })?;
        drop(paths_c);
        Ok(Self { store, state, ctx })
    }

    pub fn eval(&mut self, expr: &str) -> Result<Value> {
        self.eval_with_path(".", expr)
    }

    pub fn eval_with_path(&mut self, path: &str, expr: &str) -> Result<Value> {
        let expr = CString::new(expr).map_err(|_| Error::UnknownError)?;
        let path = CString::new(path).map_err(|_| Error::UnknownError)?;
        let value = Value::new(&self)?;
        self.ctx.exec(|ctx| unsafe {
            nix_expr_eval_from_string(ctx, self.state, expr.as_ptr(), path.as_ptr(), value.value);
        })?;
        Ok(value)
    }

    pub fn store_path(&mut self, path: &str) -> Result<StorePath> {
        StorePath::new(&self.store, path)
    }

    pub fn build(&mut self, value: &Value) -> Result<HashMap<String, String>> {
        let ValueType::Attrs = value.get_type()? else {
            return Err(Error::NotDerivation);
        };
        let store_path = value.attrs_get(&self, "drvPath")?;
        let ValueType::String = store_path.get_type()? else {
            return Err(Error::NotDerivation);
        };
        let store_path = StorePath::new(&self.store, store_path.string()?.as_str())?;
        let mut map = HashMap::<String, String>::new();
        self.ctx.exec(|ctx| unsafe {
            nix_store_realise(
                ctx,
                self.store.store,
                store_path.store_path,
                &mut map as *mut _ as *mut _,
                Some(build_cb),
            );
        })?;
        Ok(map)
    }
}

impl Drop for State {
    fn drop(&mut self) {
        unsafe { nix_state_free(self.state) };
    }
}
