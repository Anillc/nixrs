use context::Context;
use nixrs_sys::{nix_libexpr_init, nix_libstore_init, nix_libutil_init};
use utils::Result;

pub mod utils;
pub mod context;
pub mod store;
pub mod state;
pub mod value;
pub mod path;

pub fn init() -> Result<()> {
  let ctx = Context::new();
  unsafe {
    nix_libutil_init(ctx.ctx);
    ctx.check()?;
    nix_libstore_init(ctx.ctx);
    ctx.check()?;
    nix_libexpr_init(ctx.ctx);
    ctx.check()?;
  }
  Ok(())
}
