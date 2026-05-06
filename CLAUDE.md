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

The entire application lives in `src/main.rs`. It follows the standard ratatui event-loop pattern:

1. **`main()`** — reads `tui_config.json`, sets up the crossterm terminal (raw mode, alternate screen, mouse capture), calls `run_app()`, then tears down the terminal on exit.
2. **`run_app()`** — the event loop: draws a frame, blocks on `event::read()`, exits when `q` is pressed.
3. **`ui()`** — renders a single full-screen `Paragraph` widget inside an optional bordered `Block`, built from the config's content items.

**Config-driven rendering**: `tui_config.json` (loaded at runtime from the current working directory) drives everything displayed. It deserializes into `TuiConfig` (title, border, margin, content list) and `ContentItem` (type, text, optional color/style). `parse_color()` and `parse_style_modifiers()` translate the JSON string values into ratatui `Color` and `Modifier` types.

**Dependencies**: `ratatui` for TUI widgets/layout, `crossterm` as the terminal backend (re-exported through ratatui), `serde`/`serde_json` for config deserialization.
