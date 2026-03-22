# Preview Architecture Guide

## Goal

The preview system is no longer a hardcoded demo string. It is now a reusable subsystem that can host multiple preview modes inside the same `Preview` panel while keeping the shared business state independent from any specific renderer or host process implementation.

The current implementation is optimized for:

- one shared Rust core
- multiple preview modes presented as panel-local tabs
- TUI runtime ownership of external processes
- future expansion to GUI and native-shell hosts without rewriting theme logic

## Core Model

The shared preview model lives in [preview.rs](/Users/ysl/Code/theme/src/preview.rs).

The important types are:

- `PreviewMode`
  - stable mode identity
  - current modes are `Code`, `Shell`, and `Lazygit`
- `PreviewState`
  - active preview mode
  - whether preview input capture is active
  - current runtime frame
  - current runtime status text
- `PreviewFrame`
  - the rendered content contract
  - can be `Document`, `Placeholder`, or `Error`
- `PreviewDocument`
  - renderer-facing structured text document
  - line-based, span-based, styled, and platform-neutral
- `PreviewRuntimeEvent`
  - runtime-to-core messages such as frame updates and process exit

The important architectural choice is that the preview panel does not consume raw PTY bytes and does not know about child-process details. It consumes `PreviewFrame`.

## Why `PreviewFrame` Exists

The preview system has to support at least two different content sources:

- built-in semantic sample documents
- runtime-backed terminal sessions

Those sources are fundamentally different, but the panel only needs one stable rendering contract. `PreviewFrame` is that contract.

This is why the view layer does not special-case:

- sample code
- terminal output
- future embedded-editor output

They all normalize into a document/message frame first.

## View Integration

The preview panel is built in [theme_tab.rs](/Users/ysl/Code/theme/src/app/view/theme_tab.rs).

Two view-layer changes make the panel reusable:

- `PanelView.tabs`
  - lets any panel expose panel-local tabs
  - preview modes are rendered here
- `PanelView.header_lines`
  - lets any panel expose lightweight mode/status copy above the body

The body itself now uses the generic document view type in [types.rs](/Users/ysl/Code/theme/src/app/view/types.rs), not a preview-specific or code-specific widget name.

That means the current preview panel shape is:

- title
- mode tabs
- header/status line
- document body

This keeps the panel composable inside the existing `MainWindow -> Workspace -> Panel` hierarchy.

## Runtime Boundary

The TUI runtime owns external preview sessions in [preview.rs](/Users/ysl/Code/theme/src/platform/tui/preview.rs).

This code is intentionally not in core because it depends on:

- PTY allocation
- subprocess lifecycle
- terminal resizing
- byte-stream parsing
- direct key forwarding

Those are runtime responsibilities, not business-state responsibilities.

The current runtime flow is:

1. Core selects a `PreviewMode`
2. TUI runtime notices a runtime-backed mode is active
3. TUI runtime starts or resizes the hosted session
4. PTY bytes are parsed into a terminal screen model
5. The screen model is converted into `PreviewDocument`
6. Runtime dispatches `PreviewRuntimeEvent`
7. Core stores the new `PreviewFrame`
8. View layer renders the same `Preview` panel using that frame

So the contract is:

- runtime owns processes
- core owns preview state
- view consumes normalized frames

## Input Capture Model

Interactive preview modes need direct keyboard input. That cannot be handled by the normal application keymap alone, because a child terminal program must receive raw keystrokes.

The current model is:

- normal mode
  - keys go through the normal `Event -> Intent` mapping
  - left/right on the focused preview panel switch preview tabs
  - enter activates capture
- capture mode
  - TUI runtime intercepts keys first
  - keys are encoded and forwarded to the PTY
  - `Ctrl+G` releases capture and returns control to the main app

This split is intentional. The normal app event system remains semantic, while capture mode temporarily delegates raw input to the preview host.

## Why Panel-Local Tabs

Preview mode switching is represented as tabs inside the preview panel rather than workspace tabs because preview modes are not separate workspaces. They are alternate presentations of the same preview surface.

That distinction matters:

- workspace tabs change the overall tool context
- preview tabs change only how the preview panel is sourced

This keeps layout and state ownership clean.

## Built-In Sample vs Runtime-Backed Preview

The current system already supports both classes:

- `Code`
  - built in
  - deterministic semantic preview
  - no process lifecycle
- `Shell`
  - runtime-backed PTY session
  - interactive
- `Lazygit`
  - runtime-backed PTY session
  - interactive

All three are shown through the same panel chrome and the same `PreviewFrame` rendering path.

## Why This Architecture Generalizes

This structure is designed so that future providers do not require a new preview panel type.

A future provider only needs to produce the same normalized output:

- a `PreviewFrame::Document`
- or a `PreviewFrame::Placeholder`
- or a `PreviewFrame::Error`

That means future work can add:

- `nvim --embed` provider
- alternate sample scenes
- non-interactive ANSI transcript previews
- GUI-hosted preview sessions

without redesigning the panel.

## Current Limits

The current implementation is intentionally minimal in a few places:

- GUI does not yet host external preview processes
- runtime preview providers are hardcoded, not yet user-configurable
- capture mode uses a fixed release binding through the keymap preset layer
- PTY preview currently maps terminal colors into the theme palette heuristically rather than reproducing every terminal feature

These are acceptable limitations for the current stage because the architecture already isolates the right seams.

## Recommended Future Direction

If preview complexity grows, keep these rules:

- keep PTY/editor process management in the runtime layer
- keep `PreviewState` and `PreviewFrame` platform-neutral
- keep preview panel chrome generic
- add providers by normalizing them into `PreviewFrame`, not by adding renderer-specific exceptions
- treat input capture as a host/runtime concern, not as a generic core event path

That keeps the preview subsystem consistent with the rest of the app architecture:

- shared Rust core for state and domain logic
- renderer/runtime adapters for platform behavior
- reusable view contracts between them
