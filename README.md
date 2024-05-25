# nixrs

[Nix](https://github.com/NixOS/nix) binding for Rust

# example

See [example](./example)

```rust
fn eval() -> Result<()> {
  init()?;
  let mut state = State::new(Store::new("daemon")?)?;
  let libclang = &state.eval("(import <nixpkgs> {}).libclang")?;
  dbg!(state.build(libclang)?);
  Ok(())
}
```
