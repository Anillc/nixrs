# nixrs

[Nix](https://github.com/NixOS/nix) binding for [Rust](https://rust-lang.org)

## Examples

See [examples](./nixrs/examples)

```shell
cd nixrs && cargo run --example eval
```

```rust
use nixrs::{init, state::State, store::Store};

fn main() -> anyhow::Result<()> {
    init()?;
    let mut state = State::new(Store::new("daemon")?)?;
    let libclang = &state.eval("(import <nixpkgs> {}).libclang")?;
    dbg!(state.build(libclang)?);
    Ok(())
}
```
