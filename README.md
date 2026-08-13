# Vertical Sidebar

A small Zellij WebAssembly plugin example that renders the current username and clickable tab list in a narrow left sidebar.

## Build

```sh
cargo build
```

The debug plugin artifact is:

```text
target/wasm32-wasip1/debug/vertical-sidebar.wasm
```

## Load In A New Zellij Session

From this repository:

```sh
cargo build
zellij -s vertical-sidebar-demo -n layouts/dev.kdl
```

The debug layout renders the current username in the left sidebar.

## Reload In An Existing Zellij Session

```sh
zellij action start-or-reload-plugin file:target/wasm32-wasip1/debug/vertical-sidebar.wasm
```

## Release

GitHub Actions builds and uploads the release WASM when a version tag is pushed:

```sh
git tag v0.1.0
git push origin v0.1.0
```

The release assets are:

```text
vertical-sidebar.wasm
SHA256SUMS
```

## Debugging

Plugin `STDERR` is written to the Zellij log. After starting the layout, inspect it with:

```sh
zellij setup --check
tail -f /tmp/zellij-$(id -u)/zellij-log/zellij.log
```

For heavier terminal/debug output, start Zellij with:

```sh
zellij --debug --session vertical-sidebar-demo --layout layouts/dev.kdl
```

## Development Notice

This project was developed with LLM-assisted coding. Human review and local testing were used to validate the generated changes.
