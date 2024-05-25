use context::Context;
use nixrs_sys::{nix_libexpr_init, nix_libstore_init, nix_libutil_init};
use utils::Result;

use crate::{state::State, store::Store};

mod utils;
mod context;
mod store;
mod state;
mod value;

fn main() {
  init().unwrap();
  let mut state = State::new(Store::new("daemon").unwrap()).unwrap();
  let two = state.eval("{ a = 114; b = 514; }").unwrap().attrs_len().unwrap();
  dbg!(two);
}

pub fn init() -> Result<()> {
  let mut ctx = Context::new();
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
