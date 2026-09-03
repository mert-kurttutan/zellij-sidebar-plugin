# AGENTS.md

## Project Intent

Build a Zellij WebAssembly plugin that provides a vertical sidebar for terminal workspace navigation and status.

## Zellij Plugin Basics

- Zellij plugins are WASI/WebAssembly programs loaded as panes.
- Rust plugins should use the `zellij-tile` crate and import the SDK with `use zellij_tile::prelude::*;`.
- A plugin keeps state in a struct, implements `ZellijPlugin`, and registers it with `register_plugin!`.
- Core lifecycle methods:
  - `load(&mut self, configuration: BTreeMap<String, String>)`: initialize state, read plugin configuration, request permissions, and subscribe to events.
  - `update(&mut self, event: Event) -> bool`: react to subscribed events; return `true` when the UI should re-render.
  - `pipe(&mut self, pipe_message: PipeMessage) -> bool`: handle messages sent through Zellij pipes; return `true` when UI changes.
  - `render(&mut self, rows: usize, cols: usize)`: draw the current UI for the pane content area.

## Rendering Guidance

- `render` output is written to STDOUT and interpreted as UTF-8 ANSI by Zellij.
- Zellij clears the previous render each time `render` runs.
- The `rows` and `cols` passed to `render` are the plugin content size, excluding surrounding pane frames.
- Always design for narrow panes; use `saturating_sub`, truncate text, and avoid assuming minimum dimensions.
- Prefer Zellij's built-in UI components from `zellij-tile` where useful:
  - `Text`
  - `Table`
  - `Ribbon`
  - `NestedList`
  - `print_text_with_coordinates`
  - `print_table_with_coordinates`
- Built-in component color indexes are theme-aware; prefer them over hard-coded ANSI colors.

## Events And State

- Subscribe in `load` with `subscribe(&[EventType::...])`.
- Common sidebar-relevant events:
  - `TabUpdate`: tabs, active tab, tab names.
  - `PaneUpdate`: panes and focus state.
  - `ModeUpdate`: current input mode and palette/theme info.
  - `Key`: keyboard input when the plugin is focused.
  - `PermissionRequestResult`: permission grant/deny results.
- Events are asynchronous and should not be assumed to arrive in a fixed order.
- Keep the render path deterministic from plugin state; update state in `update` and `pipe`.

## Permissions

- Request permissions in `load` with `request_permission(&[PermissionType::...])`.
- Subscribe to `PermissionRequestResult` if behavior depends on granted permissions.
- Use the smallest permission set needed.
- Likely permissions for a vertical sidebar:
  - `ReadApplicationState`: read tabs, panes, UI state.
  - `ChangeApplicationState`: focus tabs or panes if the sidebar is interactive.
  - `RunActionsAsUser`: only if executing user-style Zellij actions is necessary.
  - `InterceptInput`: only if the sidebar must capture keys while not focused.

## Configuration And Loading

- Plugin configuration is passed as key/value data and received by `load`.
- Layout example for a local build:

```kdl
layout {
    pane split_direction="vertical" {
        pane size=24 borderless=true {
            plugin location="file:/absolute/path/to/vertical-sidebar.wasm"
        }
        pane
    }
}
```

- Plugin URL forms include:
  - `file:/absolute/path/to/plugin.wasm`
  - `zellij:built-in-plugin`
  - `https://.../plugin.wasm`
  - bare aliases configured in Zellij's `plugins` block.
- Local development reload command:

```sh
zellij action start-or-reload-plugin file:target/wasm32-wasi/debug/vertical-sidebar.wasm
```

## Development Environment

- This repository's Rust tooling and WASM target are provided by `flake.nix`.
- Run Cargo, Zellij, and other development commands inside the shell: `nix develop --command <command>` (or enter it first with `nix develop`).
- Do not invoke the host toolchain for project build or verification commands.

## Build And Distribution

- Build debug artifacts with `cargo build`.
- Build release artifacts with `cargo build --release`.
- Expected artifact path:

```text
target/wasm32-wasi/release/<plugin-name>.wasm
```

- Release builds should optimize WASM size and startup:

```toml
[profile.release]
lto = true
strip = true
codegen-units = 1
```

## Vertical Sidebar Design Notes

- Treat the sidebar as a narrow, persistent pane.
- Make it useful at 20-30 columns wide.
- Render dense status/navigation rather than explanatory copy.
- Initial feature target:
  - active session or mode indicator
  - tab list with active tab marker
  - focused pane indicator
  - compact key hints only when focused and space allows
- Avoid blocking work in `update` or `render`; use `ZellijWorker` only for genuinely long-running work.

## Primary References

- Zellij plugin overview: https://zellij.dev/documentation/plugins.html
- Plugin development: https://zellij.dev/documentation/plugin-development
- Rust plugin tutorial: https://zellij.dev/tutorials/developing-a-rust-plugin/
- Rust SDK docs: https://docs.rs/zellij-tile/latest/zellij_tile/
- Plugin API: https://zellij.dev/documentation/plugin-api
- Commands: https://zellij.dev/documentation/plugin-api-commands
- Permissions: https://zellij.dev/documentation/plugin-api-permissions
- Rendering UI: https://zellij.dev/documentation/plugin-ui-rendering.html
- Loading plugins: https://zellij.dev/documentation/plugin-loading.html
- Plugin configuration: https://zellij.dev/documentation/plugin-api-configuration.html
- Layouts: https://zellij.dev/documentation/creating-a-layout.html
