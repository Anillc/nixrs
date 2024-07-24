use nixrs_sys::{nix_libexpr_init, nix_libstore_init, nix_libutil_init};

mod context;
mod path;
mod state;
mod store;
mod utils;
mod value;

pub use self::{
    path::StorePath,
    state::State,
    store::Store,
    utils::Error,
    value::{Value, ValueType},
};

pub fn init() -> Result<(), Error> {
    let ctx = context::Context::new();
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
