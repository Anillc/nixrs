use nixrs_sys::{nix_c_context, nix_c_context_create, nix_c_context_free};

pub struct Context {
  pub(crate) ctx: *mut nix_c_context,
}

impl Context {
  pub fn new() -> Context {
    let ctx = unsafe { nix_c_context_create() };
    Context { ctx }
  }
}

impl Drop for Context {
  fn drop(&mut self) {
    unsafe { nix_c_context_free(self.ctx) };
  }
}
