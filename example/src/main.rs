use anyhow::{Ok, Result};
use nixrs::{init, state::State, store::Store};

fn eval() -> Result<()> {
  init()?;
  let mut state = State::new(Store::new("daemon")?)?;
  let libclang = &state.eval("(import <nixpkgs> {}).libclang")?;
  dbg!(state.build(libclang)?);
  Ok(())
}

fn main() {
  eval().unwrap();
}
