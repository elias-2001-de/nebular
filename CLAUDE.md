# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Nebular is a Rust-based TUI (Terminal User Interface) browser. The long-term goal is to define a simpler web standard that only targets the terminal, with Lua scripting support. Currently it is an early prototype.

## Commands

```bash
cargo build          # compile
cargo run            # run (requires tui_config.json in working directory)
cargo check          # fast compile check without producing a binary
cargo clippy         # lint
cargo fmt            # format
cargo test           # run tests
```

## Architecture

The application is split across two files:

- **`src/dsl.rs`** — parses `.neb` files into `TuiConfig`/`ContentItem` structs.
- **`src/main.rs`** — terminal setup, the ratatui event loop, and rendering.

**Event-loop flow**: `main()` loads `page.neb`, sets up the crossterm terminal (raw mode, alternate screen, mouse capture), calls `run_app()`, then tears down the terminal on exit. `run_app()` draws a frame per iteration and exits on `q`. `ui()` renders a single full-screen `Paragraph` inside an optional bordered `Block`.

**Dependencies**: `ratatui` for widgets/layout, `crossterm` as the terminal backend (re-exported through ratatui).

## DSL (`.neb` files)

Pages are defined in `page.neb` (loaded from the working directory at startup). The DSL uses a Maud-inspired block syntax parsed by a tokenizer + recursive-descent parser in `src/dsl.rs`.

```
// comment
page title="My App" border margin=1 {
    .blue.bold  "Hello!"        // color + modifier(s) then quoted text
    .green      "Sub-text"      // color only
    ""                          // blank line in output
    "Plain unstyled text"       // no modifiers = white
}
```

**Page attributes** (all optional): `title="…"`, `border` (flag), `margin=N`.  
**Supported colors**: `red green blue yellow cyan magenta gray grey white`.  
**Supported modifiers**: `bold italic underlined` — chain freely: `.cyan.bold.italic`.
