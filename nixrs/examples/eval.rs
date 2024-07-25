use nixrs::{init, state::State, store::Store};

fn main() -> anyhow::Result<()> {
    init()?;
    let mut state = State::new(Store::new("daemon")?)?;
    let libclang = &state.eval("(import <nixpkgs> {}).libclang")?;
    dbg!(state.build(libclang)?);
    Ok(())
}
