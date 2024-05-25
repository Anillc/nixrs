use context::Context;
use nixrs_sys::{nix_libexpr_init, nix_libstore_init, nix_libutil_init};
use utils::{NixRSError, Result};

use crate::{state::State, store::Store};

mod utils;
mod context;
mod store;
mod state;
mod value;

fn main() {
  init().unwrap();
  let mut state = State::new(Store::new("daemon").unwrap()).unwrap();
  let two = state.eval("1 + 1").unwrap().int().unwrap();
  dbg!(two);
}

pub fn init() -> Result<()> {
  let ctx = Context::new();
  unsafe {
    nix_libutil_init(ctx.ctx);
    NixRSError::from_raw(&ctx)?;
    nix_libstore_init(ctx.ctx);
    NixRSError::from_raw(&ctx)?;
    nix_libexpr_init(ctx.ctx);
    NixRSError::from_raw(&ctx)?;
  }
  Ok(())
}
