use std::{collections::HashMap, ptr::null_mut};

use std::ffi::CString;
use nixrs_sys::{nix_alloc_value, nix_bindings_builder_free, nix_bindings_builder_insert, nix_gc_decref, nix_get_attr_byidx, nix_get_attr_byname, nix_get_attrs_size, nix_get_bool, nix_get_float, nix_get_int, nix_get_list_byidx, nix_get_list_size, nix_get_path_string, nix_get_string, nix_get_type, nix_init_apply, nix_init_bool, nix_init_float, nix_init_int, nix_init_null, nix_init_path_string, nix_init_string, nix_list_builder_free, nix_list_builder_insert, nix_make_attrs, nix_make_bindings_builder, nix_make_list, nix_make_list_builder, ValueType_NIX_TYPE_ATTRS, ValueType_NIX_TYPE_BOOL, ValueType_NIX_TYPE_EXTERNAL, ValueType_NIX_TYPE_FLOAT, ValueType_NIX_TYPE_FUNCTION, ValueType_NIX_TYPE_INT, ValueType_NIX_TYPE_LIST, ValueType_NIX_TYPE_NULL, ValueType_NIX_TYPE_PATH, ValueType_NIX_TYPE_STRING, ValueType_NIX_TYPE_THUNK};

use crate::utils::string_from_c;
use crate::{context::Context, state::State, utils::{NixRSError, Result}};


#[derive(Debug, Clone, Copy)]
pub enum ValueType {
  Thunk, Int, Float, Bool, String, Path,
  Null, Attrs, List, Function, External,
}

impl ValueType {
  #[allow(non_upper_case_globals)]
  pub(crate) fn from_raw(ty: libc::c_uint) -> Result<ValueType> {
    match ty {
      ValueType_NIX_TYPE_THUNK => Ok(Self::Thunk),
      ValueType_NIX_TYPE_INT => Ok(Self::Int),
      ValueType_NIX_TYPE_FLOAT => Ok(Self::Float),
      ValueType_NIX_TYPE_BOOL => Ok(Self::Bool),
      ValueType_NIX_TYPE_STRING => Ok(Self::String),
      ValueType_NIX_TYPE_PATH => Ok(Self::Path),
      ValueType_NIX_TYPE_NULL => Ok(Self::Null),
      ValueType_NIX_TYPE_ATTRS => Ok(Self::Attrs),
      ValueType_NIX_TYPE_LIST => Ok(Self::List),
      ValueType_NIX_TYPE_FUNCTION => Ok(Self::Function),
      ValueType_NIX_TYPE_EXTERNAL => Ok(Self::External),
      _ => Err(crate::utils::NixRSError::UnknownError),
    }
  }
}

#[derive(Debug)]
pub struct Value {
  ctx: Context,
  pub(crate) value: *mut nixrs_sys::Value,
}

impl Value {
  pub fn new(state: &State) -> Result<Value> {
    let mut ctx = Context::new();
    let value = unsafe {
      let value = nix_alloc_value(ctx.ctx, state.state);
      ctx.check()?;
      value
    };
    Ok(Value { ctx, value })
  }

  pub(crate) unsafe fn from_raw(value: *mut nixrs_sys::Value) -> Value {
    Value { ctx: Context::new(), value }
  }

  pub fn get_type(&mut self) -> Result<ValueType> {
    let ty = unsafe {
      let ty = nix_get_type(self.ctx.ctx, self.value);
      self.ctx.check()?;
      ty
    };
    ValueType::from_raw(ty)
  }

  pub fn init_thunk(&mut self, f: &Value, arg: &Value) -> Result<()> {
    unsafe {
      nix_init_apply(self.ctx.ctx, self.value, f.value, arg.value);
      self.ctx.check()?;
    };
    Ok(())
  }

  pub fn init_int(&mut self, value: i64) -> Result<()> {
    unsafe {
      nix_init_int(self.ctx.ctx, self.value, value);
      self.ctx.check()?;
    };
    Ok(())
  }

  pub fn init_float(&mut self, value: f64) -> Result<()> {
    unsafe {
      nix_init_float(self.ctx.ctx, self.value, value);
      self.ctx.check()?;
    };
    Ok(())
  }

  pub fn init_bool(&mut self, value: bool) -> Result<()> {
    unsafe {
      nix_init_bool(self.ctx.ctx, self.value, value);
      self.ctx.check()?;
    };
    Ok(())
  }

  pub fn init_string(&mut self, value: &str) -> Result<()> {
    let value = CString::new(value).map_err(|_| NixRSError::UnknownError)?;
    unsafe {
      nix_init_string(self.ctx.ctx, self.value, value.as_ptr());
      self.ctx.check()?;
    };
    drop(value);
    Ok(())
  }

  pub fn init_path(&mut self, state: &State, value: &str) -> Result<()> {
    let value = CString::new(value).map_err(|_| NixRSError::UnknownError)?;
    unsafe {
      nix_init_path_string(self.ctx.ctx, state.state, self.value, value.as_ptr());
      self.ctx.check()?;
    };
    drop(value);
    Ok(())
  }

  pub fn init_null(&mut self) -> Result<()> {
    unsafe {
      nix_init_null(self.ctx.ctx, self.value);
      self.ctx.check()?;
    };
    Ok(())
  }

  pub fn init_attrs(&mut self, state: &State, values: HashMap<String, &Value>) -> Result<()> {
    unsafe {
      let builder = nix_make_bindings_builder(self.ctx.ctx, state.state, values.len());
      self.ctx.check()?;
      for (name, value) in values {
        let name = match CString::new(name) {
          Ok(name) => name,
          Err(_) => {
            nix_bindings_builder_free(builder);
            return Err(NixRSError::UnknownError);
          }
        };
        nix_bindings_builder_insert(self.ctx.ctx, builder, name.as_ptr(), value.value);
        drop(name);
        if let err@Err(_) = self.ctx.check() {
          nix_bindings_builder_free(builder);
          return err;
        }
      }
      nix_make_attrs(self.ctx.ctx, self.value, builder);
      nix_bindings_builder_free(builder);
      self.ctx.check()?;
    };
    Ok(())
  }

  pub fn init_list(&mut self, state: &State, values: &[&Value]) -> Result<()> {
    unsafe {
      let builder = nix_make_list_builder(self.ctx.ctx, state.state, values.len());
      self.ctx.check()?;
      for (i, value) in values.iter().enumerate() {
        nix_list_builder_insert(self.ctx.ctx, builder, i as u32, value.value);
        if let err@Err(_) = self.ctx.check() {
          nix_list_builder_free(builder);
          return err;
        }
      }
      nix_make_list(self.ctx.ctx, builder, self.value);
      nix_list_builder_free(builder);
      self.ctx.check()?;
    };
    Ok(())
  }

  // TODO: function and external

  pub fn int(&mut self) -> Result<i64> {
    let int = unsafe {
      let int = nix_get_int(self.ctx.ctx, self.value);
      self.ctx.check()?;
      int
    };
    Ok(int)
  }

  pub fn float(&mut self) -> Result<f64> {
    let float = unsafe {
      let float = nix_get_float(self.ctx.ctx, self.value);
      self.ctx.check()?;
      float
    };
    Ok(float)
  }

  pub fn bool(&mut self) -> Result<bool> {
    let bool = unsafe {
      let bool = nix_get_bool(self.ctx.ctx, self.value);
      self.ctx.check()?;
      bool
    };
    Ok(bool)
  }

  pub fn string(&mut self) -> Result<String> {
    let string = unsafe {
      let mut vec: Vec<u8> = Vec::new();
      nix_get_string(self.ctx.ctx, self.value, Some(get_string_cb), &mut vec as *mut _ as *mut _);
      self.ctx.check()?;
      String::from_utf8(vec).map_err(|_| NixRSError::UnknownError)?
    };
    Ok(string)
  }

  pub fn path(&mut self) -> Result<String> {
    let path = unsafe {
      let path = nix_get_path_string(self.ctx.ctx, self.value);
      self.ctx.check()?;
      string_from_c(path)?
    };
    Ok(path)
  }

  pub fn attrs_len(&mut self) -> Result<usize> {
    let size = unsafe {
      let size = nix_get_attrs_size(self.ctx.ctx, self.value);
      self.ctx.check()?;
      size as usize
    };
    Ok(size)
  }

  pub fn attrs_get_byname(&mut self, state: &State, name: &str) -> Result<Value> {
    let value = unsafe {
      let name = CString::new(name).map_err(|_| NixRSError::UnknownError)?;
      let value = nix_get_attr_byname(self.ctx.ctx, self.value, state.state, name.as_ptr());
      drop(name);
      self.ctx.check()?;
      Value::from_raw(value)
    };
    Ok(value)
  }

  pub fn attrs_get_byid(&mut self, state: &State, id: usize) -> Result<(String, Value)> {
    let value = unsafe {
      let mut name: *const libc::c_char = null_mut();
      let value = nix_get_attr_byidx(self.ctx.ctx, self.value, state.state, id as u32, &mut name as *mut *const _);
      self.ctx.check()?;
      (string_from_c(name)?, Value::from_raw(value))
    };
    Ok(value)
  }

  pub fn list_len(&mut self) -> Result<usize> {
    let size = unsafe {
      let size = nix_get_list_size(self.ctx.ctx, self.value);
      self.ctx.check()?;
      size as usize
    };
    Ok(size)
  }

  pub fn list_get(&mut self, state: &State, id: usize) -> Result<Value> {
    let value = unsafe {
      let value = nix_get_list_byidx(self.ctx.ctx, self.value, state.state, id as u32);
      self.ctx.check()?;
      Value::from_raw(value)
    };
    Ok(value)
  }
}

impl Drop for Value {
  fn drop(&mut self) {
    // TODO: context?
    unsafe { nix_gc_decref(null_mut(), self.value) };
  }
}

pub unsafe extern "C" fn get_string_cb(start: *const libc::c_char, n: libc::c_uint, user_data: *mut libc::c_void) {
    let ret = user_data as *mut Vec<u8>;
    let slice = std::slice::from_raw_parts(start as *const u8, n as usize);
    (*ret).extend_from_slice(slice);
}
