# Third-Party Libraries

## Principle

This project now follows a simple dependency rule:

- Domain logic stays custom.
- Cross-cutting infrastructure should prefer mature third-party libraries.
- Hand-rolled code is reserved for product-specific behavior, not generic plumbing.

Applied to this codebase, that means:

- theme evaluation, rule editing, palette generation, and export semantics stay project-owned
- CLI parsing, color-space conversion, JSON serialization, error derivation, native compilation, standard config-directory handling, and temp-file handling use established crates

## Runtime Dependencies

### `clap`

- Version: `4.6.0`
- Role: declarative command-line parsing
- Why: replaces manual argv parsing with a well-maintained parser and built-in conflict/help handling
- Main usage: [src/platform/mod.rs](/Users/ysl/Code/theme/src/platform/mod.rs)

### `crossterm`

- Version: `0.28.1`
- Role: terminal input/output backend
- Why: portable TUI event and terminal control layer
- Main usage: TUI runtime and event adapter

### `directories`

- Version: `6.0.0`
- Role: standard OS config-directory discovery
- Why: editor-only local configuration should live in the platform's normal config location, not in ad-hoc hardcoded paths
- Main usage: [src/persistence/editor_config.rs](/Users/ysl/Code/theme/src/persistence/editor_config.rs)

### `fluent-bundle`

- Version: `0.16.0`
- Role: Fluent message argument/value model
- Why: supports parameterized UI translations without hand-rolling interpolation logic
- Main usage: [src/i18n/mod.rs](/Users/ysl/Code/theme/src/i18n/mod.rs)

### `fluent-templates`

- Version: `0.13.2`
- Role: static Fluent resource loading
- Why: provides declarative `.ftl`-based i18n resources for shared TUI/GUI UI copy
- Main usage:
  - [src/i18n/mod.rs](/Users/ysl/Code/theme/src/i18n/mod.rs)
  - [locales/en-US/ui.ftl](/Users/ysl/Code/theme/locales/en-US/ui.ftl)
  - [locales/zh-CN/ui.ftl](/Users/ysl/Code/theme/locales/zh-CN/ui.ftl)

### `palette`

- Version: `0.7.6`
- Role: color-space conversion and color operations
- Why: replaces handwritten HSL/RGB conversion math with a dedicated color-management crate that is designed around correctness and supports deeper color work later
- Feature choice: disables default features and keeps only `std`, so the app avoids unused named-color parsing dependencies
- Main usage: [src/color.rs](/Users/ysl/Code/theme/src/color.rs)

### `ratatui`

- Version: `0.29.0`
- Role: TUI rendering framework
- Why: retained project choice for terminal layout/rendering
- Main usage: [src/platform/tui/renderer.rs](/Users/ysl/Code/theme/src/platform/tui/renderer.rs)

### `serde`

- Version: `1.0.228`
- Role: shared serialization model
- Why: derive-based data serialization across project files and GUI snapshot models
- Main usage: project persistence and snapshot structs

### `serde_json`

- Version: `1.0.149`
- Role: GUI snapshot JSON serialization
- Why: replaces hand-written JSON string assembly in the AppKit bridge
- Main usage: [src/platform/gui/bridge.rs](/Users/ysl/Code/theme/src/platform/gui/bridge.rs)

### `toml`

- Version: `0.9.8`
- Role: project file format serialization
- Why: project files are intentionally human-editable and TOML fits that requirement well
- Main usage: [src/persistence/project_file.rs](/Users/ysl/Code/theme/src/persistence/project_file.rs)

### `thiserror`

- Version: `2.0.18`
- Role: error type derivation
- Why: replaces repetitive manual `Display`/`Error` impls for infrastructure-facing error enums
- Main usage:
  - [src/platform/mod.rs](/Users/ysl/Code/theme/src/platform/mod.rs)
  - [src/export/mod.rs](/Users/ysl/Code/theme/src/export/mod.rs)
  - [src/persistence/project_file.rs](/Users/ysl/Code/theme/src/persistence/project_file.rs)
  - [src/evaluator.rs](/Users/ysl/Code/theme/src/evaluator.rs)
  - [src/app/state.rs](/Users/ysl/Code/theme/src/app/state.rs)

### `unic-langid`

- Version: `0.9.6`
- Role: locale identifiers
- Why: keeps supported editor locales explicit and interoperable with Fluent resources
- Main usage:
  - [src/persistence/editor_config.rs](/Users/ysl/Code/theme/src/persistence/editor_config.rs)
  - [src/i18n/mod.rs](/Users/ysl/Code/theme/src/i18n/mod.rs)

## Build Dependencies

### `cc`

- Version: `1.2.0`
- Role: native code compilation in `build.rs`
- Why: replaces direct `clang`/`ar` shell orchestration with Cargo-standard native build integration
- Main usage: [build.rs](/Users/ysl/Code/theme/build.rs)

## Dev Dependencies

### `tempfile`

- Version: `3.24.0`
- Role: temporary files in tests
- Why: replaces ad-hoc temp-path naming and manual cleanup
- Main usage:
  - [src/export/template.rs](/Users/ysl/Code/theme/src/export/template.rs)
  - [src/persistence/project_file.rs](/Users/ysl/Code/theme/src/persistence/project_file.rs)

## Platform / System Dependencies

These are not Cargo crates, but they are still external dependencies:

- macOS `Cocoa/AppKit` frameworks for the native GUI backend
- system toolchain support required to compile Objective-C code through Cargo build scripts

## Deliberately Not Abstracted Into Third-Party Libraries

The following areas remain custom on purpose:

- theme parameter model
- palette generation
- theme-specific color heuristics and palette slot formulas
- rule graph and evaluator
- snapshot/view-model shape
- exporter semantics
- TUI/GUI interaction model for this product

Those are product-specific and are part of the app's core value, so replacing them with generic libraries would reduce clarity rather than improve maintainability.

## Selection Guidelines Going Forward

When adding a new dependency, it should satisfy most of these:

- solves generic infrastructure, not product logic
- widely used and actively maintained
- good docs and predictable API surface
- integrates cleanly with Rust derive/trait ecosystem
- reduces handwritten boilerplate or platform fragility
- does not meaningfully bloat the runtime model for little gain
