### Formatting
The project uses unstable `rustfmt` settings, so the formatting has to be done with nightly.
```console
cargo +nightly fmt
```

### Testing
The tests use optional dependencies, so `--all-features` is needed. The tests use `elm repl` so Elm has to be installed.

```console
cargo test --all-features
```
