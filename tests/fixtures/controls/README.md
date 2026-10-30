# Fixture roots for `relearn controls`

Synthetic roots that `tests/resolve.rs` supplies with `--root`. Nothing here is
compiled or run: the `.rs` files sit below `tests/`'s top level, so Cargo never
builds them, and the shell scripts are read, not executed. Each file exists to
put one verdict on the table — see the test that names it.
