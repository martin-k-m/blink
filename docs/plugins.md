# Plugins

Blink's plugin system follows the same convention Cargo and Git use for
their subcommands: a plugin is any executable named `blink-<name>`, and
running `blink <name>` for a command Blink doesn't recognize execs that
executable, forwarding every remaining argument and inheriting
stdin/stdout/stderr.

There's no dynamic loading (no `dlopen`/`libloading`, no in-process plugin
ABI) and no remote plugin registry. Both are deliberate:

- Dynamic loading of native code across a plugin ABI is inherently
  unsafe and fragile across Rust compiler/std versions — a real
  reliability cost for a "just works" developer tool.
- A registry needs backing infrastructure (hosting, a package format,
  namespace/security policy) that doesn't exist yet. Rather than fake
  one, `blink plugins install` only ever copies a local file you already
  have.

The subprocess convention avoids both problems while still giving a
plugin full access to build whatever it wants — a different language
entirely, if you want.

## Where Blink looks for plugins

In order:

1. `~/.blink/plugins/blink-<name>` (Blink's managed plugin directory).
2. Every directory on `PATH`, searched in `PATH` order.

`blink plugins` lists everything discovered in both locations.

## Installing a plugin

```sh
blink plugins install ./my-tool --name mytool
# copies ./my-tool to ~/.blink/plugins/blink-mytool
blink mytool --some-arg
# runs it
```

You can also just put a `blink-<name>` executable anywhere on `PATH`
yourself (e.g. via `cargo install`, if the plugin is a Cargo crate with a
`[[bin]] name = "blink-<name>"`) — Blink doesn't need to know about it
ahead of time.

## Writing a plugin

A plugin is just a program. It receives Blink's remaining CLI arguments as
its own `argv`, and its stdout/stderr/exit code are Blink's — there's no
required protocol, no manifest, no metadata file. A minimal Rust plugin:

```rust
// src/main.rs, with [[bin]] name = "blink-hello" in Cargo.toml
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    println!("hello from a blink plugin, args: {args:?}");
}
```

If your plugin wants to understand the project it's running against, it
can shell out to Blink itself for structured data:

```sh
blink analyze --json | your-plugin-logic
```

That's the intended integration point — `blink analyze/health/report
--json` are stable, documented JSON shapes (see
[`docs/analysis.md`](analysis.md)) precisely so plugins (and other
tooling) can consume them without needing to link against Blink's Rust
crates directly.
