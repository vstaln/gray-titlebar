<div align="center">
  <img alt="gray-titlebar" src="assets/icon.svg" width="120" height="120" />
  <h1>gray-titlebar</h1>
  <p><strong>Terminal title spinner while the agent works.</strong></p>
  <p>
    <a href="https://gray.alignment.id">Website</a> ·
    <a href="https://gray.alignment.id/plugins/gray-titlebar">Store</a> ·
    <a href="https://github.com/vstaln/gray-titlebar">Source</a> ·
    <a href="https://github.com/vstaln/gray">gray</a>
  </p>
  <p>
    <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/license-MIT-1c1c20?style=flat-square&labelColor=0a0a0b" /></a>
    <a href="https://www.rust-lang.org"><img alt="Built with Rust" src="https://img.shields.io/badge/built%20with-rust-1c1c20?style=flat-square&labelColor=0a0a0b&logo=rust&logoColor=d4a373" /></a>
    <a href="https://gray.alignment.id/plugins/gray-titlebar"><img alt="gray plugin" src="https://img.shields.io/badge/gray-plugin-1c1c20?style=flat-square&labelColor=0a0a0b&color=7aa2f7" /></a>
  </p>
</div>

<br/>

```bash
gray plugin install gray-titlebar
```

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

## Tags

`gray` `plugin` `titlebar` `rust`

---
Part of the [gray](https://github.com/vstaln/gray) plugin ecosystem —
the open-source AI agent harness. <https://gray.alignment.id>
