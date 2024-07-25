use std::{collections::HashMap, ffi::CString, ptr::null_mut};

use nixrs_sys::{
    nix_alloc_value, nix_bindings_builder_free, nix_bindings_builder_insert, nix_gc_decref,
    nix_get_attr_byidx, nix_get_attr_byname, nix_get_attrs_size, nix_get_bool, nix_get_float,
    nix_get_int, nix_get_list_byidx, nix_get_list_size, nix_get_path_string, nix_get_string,
    nix_get_type, nix_init_apply, nix_init_bool, nix_init_float, nix_init_int, nix_init_null,
    nix_init_path_string, nix_init_string, nix_list_builder_free, nix_list_builder_insert,
    nix_make_attrs, nix_make_bindings_builder, nix_make_list, nix_make_list_builder,
    nix_value_force, nix_value_force_deep, ValueType_NIX_TYPE_ATTRS, ValueType_NIX_TYPE_BOOL,
    ValueType_NIX_TYPE_EXTERNAL, ValueType_NIX_TYPE_FLOAT, ValueType_NIX_TYPE_FUNCTION,
    ValueType_NIX_TYPE_INT, ValueType_NIX_TYPE_LIST, ValueType_NIX_TYPE_NULL,
    ValueType_NIX_TYPE_PATH, ValueType_NIX_TYPE_STRING, ValueType_NIX_TYPE_THUNK,
};

use crate::{
    context::Context,
    state::State,
    utils::{get_string_cb, string_from_c},
    utils::{Error, Result},
};

#[derive(Debug, Clone, Copy)]
pub enum ValueType {
    Thunk,
    Int,
    Float,
    Bool,
    String,
    Path,
    Null,
    Attrs,
    List,
    Function,
    External,
}

impl ValueType {
    #[allow(non_upper_case_globals)]
    pub(crate) fn from_raw(ty: libc::c_uint) -> Result<Self> {
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
            _ => Err(Error::UnknownError),
        }
    }
}

#[derive(Debug)]
pub struct Value {
    ctx: Context,
    pub(crate) value: *mut nixrs_sys::Value,
}

impl Value {
    pub fn new(state: &State) -> Result<Self> {
        let ctx = Context::new();
        let value = ctx.exec(|ctx| unsafe { nix_alloc_value(ctx, state.state) })?;
        Ok(Self { ctx, value })
    }

    unsafe fn from_raw(value: *mut nixrs_sys::Value) -> Self {
        Self {
            ctx: Context::new(),
            value,
        }
    }

    pub fn get_type(&self) -> Result<ValueType> {
        let ty = self
            .ctx
            .exec(|ctx| unsafe { nix_get_type(ctx, self.value) })?;
        ValueType::from_raw(ty)
    }

    pub fn force(&mut self, state: &State) -> Result<()> {
        self.ctx.exec(|ctx| unsafe {
            nix_value_force(ctx, state.state, self.value);
        })
    }

    pub fn force_deep(&mut self, state: &State) -> Result<()> {
        self.ctx.exec(|ctx| unsafe {
            nix_value_force_deep(ctx, state.state, self.value);
        })
    }

    pub fn init_thunk(&mut self, f: &Value, arg: &Value) -> Result<()> {
        self.ctx.exec(|ctx| unsafe {
            nix_init_apply(ctx, self.value, f.value, arg.value);
        })
    }

    pub fn init_int(&mut self, value: i64) -> Result<()> {
        self.ctx.exec(|ctx| unsafe {
            nix_init_int(ctx, self.value, value);
        })
    }

    pub fn init_float(&mut self, value: f64) -> Result<()> {
        self.ctx.exec(|ctx| unsafe {
            nix_init_float(ctx, self.value, value);
        })
    }

    pub fn init_bool(&mut self, value: bool) -> Result<()> {
        self.ctx.exec(|ctx| unsafe {
            nix_init_bool(ctx, self.value, value);
        })
    }

    pub fn init_string(&mut self, value: &str) -> Result<()> {
        let value = CString::new(value).map_err(|_| Error::UnknownError)?;
        self.ctx.exec(|ctx| unsafe {
            nix_init_string(ctx, self.value, value.as_ptr());
        })?;
        drop(value);
        Ok(())
    }

    pub fn init_path(&mut self, state: &State, value: &str) -> Result<()> {
        let value = CString::new(value).map_err(|_| Error::UnknownError)?;
        self.ctx.exec(|ctx| unsafe {
            nix_init_path_string(ctx, state.state, self.value, value.as_ptr());
        })?;
        drop(value);
        Ok(())
    }

    pub fn init_null(&mut self) -> Result<()> {
        self.ctx.exec(|ctx| unsafe {
            nix_init_null(ctx, self.value);
        })
    }

    pub fn init_attrs(&mut self, state: &State, values: HashMap<String, &Value>) -> Result<()> {
        let builder = self
            .ctx
            .exec(|ctx| unsafe { nix_make_bindings_builder(ctx, state.state, values.len()) })?;
        for (name, value) in values {
            unsafe {
                let Ok(name) = CString::new(name) else {
                    nix_bindings_builder_free(builder);
                    return Err(Error::UnknownError);
                };
                let result = self.ctx.exec(|ctx| {
                    nix_bindings_builder_insert(ctx, builder, name.as_ptr(), value.value);
                });
                drop(name);
                if let Err(err) = result {
                    nix_bindings_builder_free(builder);
                    return Err(err);
                }
            }
        }
        self.ctx.exec(|ctx| unsafe {
            nix_make_attrs(ctx, self.value, builder);
            nix_bindings_builder_free(builder)
        })
    }

    pub fn init_list(&mut self, state: &State, values: &[&Value]) -> Result<()> {
        let builder = self
            .ctx
            .exec(|ctx| unsafe { nix_make_list_builder(ctx, state.state, values.len()) })?;
        for (i, value) in values.iter().enumerate() {
            self.ctx
                .exec(|ctx| unsafe {
                    nix_list_builder_insert(ctx, builder, i as u32, value.value);
                })
                .map_err(|err| {
                    unsafe {
                        nix_list_builder_free(builder);
                    }
                    err
                })?
        }
        self.ctx.exec(|ctx| unsafe {
            nix_make_list(ctx, builder, self.value);
            nix_list_builder_free(builder);
        })
    }

    // TODO: function and external

    pub fn int(&self) -> Result<i64> {
        self.ctx.exec(|ctx| unsafe { nix_get_int(ctx, self.value) })
    }

    pub fn float(&self) -> Result<f64> {
        self.ctx
            .exec(|ctx| unsafe { nix_get_float(ctx, self.value) })
    }

    pub fn bool(&self) -> Result<bool> {
        self.ctx
            .exec(|ctx| unsafe { nix_get_bool(ctx, self.value) })
    }

    pub fn string(&self) -> Result<String> {
        let mut vec: Vec<u8> = Vec::new();
        self.ctx.exec(|ctx| unsafe {
            nix_get_string(
                ctx,
                self.value,
                Some(get_string_cb),
                &mut vec as *mut _ as *mut _,
            );
        })?;
        let string = String::from_utf8(vec).map_err(|_| Error::UnknownError)?;
        Ok(string)
    }

    pub fn path(&self) -> Result<String> {
        let path = self
            .ctx
            .exec(|ctx| unsafe { nix_get_path_string(ctx, self.value) })?;
        let path = unsafe { string_from_c(path)? };
        Ok(path)
    }

    pub fn attrs_len(&self) -> Result<usize> {
        let size = self
            .ctx
            .exec(|ctx| unsafe { nix_get_attrs_size(ctx, self.value) })?;
        Ok(size as usize)
    }

    pub fn attrs_get(&self, state: &State, name: &str) -> Result<Self> {
        let name = CString::new(name).map_err(|_| Error::UnknownError)?;
        let value = self.ctx.exec(|ctx| unsafe {
            let value = nix_get_attr_byname(ctx, self.value, state.state, name.as_ptr());
            Self::from_raw(value)
        })?;
        drop(name);
        Ok(value)
    }

    pub fn attrs_get_byid(&self, state: &State, id: usize) -> Result<(String, Self)> {
        let (name, value) = self.ctx.exec(|ctx| unsafe {
            let mut name: *const libc::c_char = null_mut();
            let value = nix_get_attr_byidx(
                ctx,
                self.value,
                state.state,
                id as u32,
                &mut name as *mut *const _,
            );
            (string_from_c(name), Self::from_raw(value))
        })?;
        Ok((name?, value))
    }

    pub fn list_len(&self) -> Result<usize> {
        let size = self
            .ctx
            .exec(|ctx| unsafe { nix_get_list_size(ctx, self.value) })?;
        Ok(size as usize)
    }

    pub fn list_get(&self, state: &State, id: usize) -> Result<Self> {
        let value = self
            .ctx
            .exec(|ctx| unsafe { nix_get_list_byidx(ctx, self.value, state.state, id as u32) })?;
        Ok(unsafe { Self::from_raw(value) })
    }

    // TODO: apply
}

impl Drop for Value {
    fn drop(&mut self) {
        // TODO: context?
        unsafe { nix_gc_decref(null_mut(), self.value) };
    }
}
