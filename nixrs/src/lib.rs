use nixrs_sys::{nix_libexpr_init, nix_libstore_init, nix_libutil_init};

pub mod context;
pub mod path;
pub mod state;
pub mod store;
pub mod utils;
pub mod value;

use self::{context::Context, utils::Result};

pub fn init() -> Result<()> {
    let ctx = Context::new();
    ctx.exec(|ctx| unsafe {
        nix_libutil_init(ctx);
    })?;
    ctx.exec(|ctx| unsafe {
        nix_libstore_init(ctx);
    })?;
    ctx.exec(|ctx| unsafe {
        nix_libexpr_init(ctx);
    })
}
