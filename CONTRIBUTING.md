### Formatting
The project uses unstable `rustfmt` settings, so the formatting has to be done with nightly.
```console
cargo +nightly fmt
```

### Testing
The tests use optional dependencies, so `--all-features` is needed. At the moment the test for forms requires manual interaction by opening a browser so it's ignored.

```console
cargo test --all-features -- --include-ignored --nocapture
```
