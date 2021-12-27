### Formatting
The project uses unstable `rustfmt` settings, so the formatting has to be done with nightly.
```console
cargo +nightly fmt
```

### Testing
The tests use optional dependencies, so `--all-features` is needed. At the moment the test for forms requires manual interaction by opening a browser so it's ignored. The tests use `elm repl` so Elm has to be installed. The Elm repl doesn't appreciate being run multiple times in parallel so `--test-threads 1` is necessary as well.

```console
cargo test --all-features -- --test-threads 1
```
```console
cargo test rocket --all-features -- --ignored --nocapture
```
