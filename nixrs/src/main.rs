use context::Context;
use nixrs_sys::{nix_libexpr_init, nix_libstore_init, nix_libutil_init};
use utils::{NixRSError, Result};

use crate::{state::State, store::Store};

mod utils;
mod context;
mod store;
mod state;

fn main() {
  init().unwrap();
  let store = Store::new("daemon").unwrap();
  let state = State::new(&store).unwrap();
  dbg!(store, state);
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
