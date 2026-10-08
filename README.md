# gray-titlebar

Terminal title spinner while the agent works (OSC 2). Port of pi's titlebar-spinner extension.

Sets the terminal title (`⬡ gray — <tool>` during tool calls, `⬡ gray —
ready` when the turn ends) by writing OSC 2 to `/dev/tty` — stdout stays
pure NDJSON. Headless (no tty) → no-op.

A sidecar plugin for [gray](https://github.com/vstaln/gray), scaffolded by
[gray-account](https://github.com/vstaln/gray-account).

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
