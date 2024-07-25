use std::{collections::HashMap, ffi::CStr};

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("NIX_ERROR_UNKNOWN: {0}")]
    NixErrorUnknown(String),
    #[error("NIX_ERROR_OVERFLOW: {0}")]
    NixErrorOverflow(String),
    #[error("NIX_ERROR_KEY: {0}")]
    NixErrorKey(String),
    #[error("NIX_ERROR_NIX_ERROR: {0}")]
    NixErrorNixError(String),
    #[error("value is not a derivation")]
    NotDerivation,
    #[error("unknown error")]
    UnknownError,
}

pub(crate) unsafe fn string_from_c(ptr: *const libc::c_char) -> Result<String> {
    CStr::from_ptr(ptr)
        .to_str()
        .map(|str| str.to_string())
        .map_err(|_| Error::UnknownError)
}

pub(crate) unsafe extern "C" fn get_string_cb(
    start: *const libc::c_char,
    n: libc::c_uint,
    user_data: *mut libc::c_void,
) {
    let ret = user_data as *mut Vec<u8>;
    let slice = std::slice::from_raw_parts(start as *const u8, n as usize);
    (*ret).extend_from_slice(slice);
}

pub(crate) unsafe extern "C" fn build_cb(
    map: *mut libc::c_void,
    out: *const libc::c_char,
    path: *const libc::c_char,
) {
    let map = map as *mut HashMap<String, String>;
    let out = string_from_c(out).unwrap();
    let path = string_from_c(path).unwrap();
    (*map).insert(out, path);
}
