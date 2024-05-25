use anyhow::{Ok, Result};
use nixrs::{init, state::State, store::Store};

fn eval() -> Result<()> {
  init()?;
  let mut state = State::new(Store::new("daemon")?)?;
  let res = state.eval("114514")?.int()?;
  dbg!(res);
  Ok(())
}

fn main() {
  eval().unwrap();
}
