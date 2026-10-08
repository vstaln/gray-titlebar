<p align="center">
  <img src="assets/gray-logo.svg" alt="gray" width="96">
</p>
<h1 align="center">gray-titlebar</h1>
<p align="center">Terminal title spinner while the agent works (OSC 2).</p>
<p align="center">
  <a href="https://github.com/vstaln/gray-titlebar/blob/main/LICENSE"><img alt="MIT License" src="https://img.shields.io/badge/license-MIT-blue.svg"></a>
  <img alt="gray plugin" src="https://img.shields.io/badge/gray-plugin-7aa2f7.svg">
  <img alt="rust" src="https://img.shields.io/badge/built%20with-rust-orange.svg">
</p>

Sets the terminal title (`⬡ gray — <tool>` during tool calls, `⬡ gray —
ready` when the turn ends) by writing OSC 2 to `/dev/tty` — stdout stays
pure NDJSON. Headless (no tty) → no-op.

## Wire methods used

- `plugin/manifest`, `plugin/shutdown` (shutdown also restores a neutral title)
- `event/notify` — `pre_tool` → `⬡ gray — <tool>` (×N for repeats),
  `post_tool` → `⬡ gray — <tool> ✓/✗`, `turn_end` → `⬡ gray — ready`
- `command/run` — `/titlebar` status, `/titlebar on|off` (persisted at
  `~/.gray/titlebar/disabled`)

No capabilities required.

## Install

```sh
gray plugin install titlebar
```

## Develop

```sh
cargo test
gray account check      # entry point + manifest handshake
gray account publish    # check → build → release → publish to the gray registry
```

Bump `version` in `Cargo.toml` before each `publish`; the registry refuses to
republish a version.

---
Part of the [gray](https://github.com/vstaln/gray) plugin ecosystem —
the open-source AI agent harness. <https://gray.alignment.id>
