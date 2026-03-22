# Development Guidelines

## Core Principles

- Product logic stays in Rust.
- Generic infrastructure should prefer mature third-party libraries over handwritten plumbing.
- Platform renderers should consume shared state/snapshot data and should not own business rules.

## UI Localization Rule

This project now has a formal i18n boundary.

- Localized scope: UI copy only.
- Current locales: `en-US` and `zh-CN`.
- Translation resources live in declarative Fluent files under [locales/en-US/ui.ftl](/Users/ysl/Code/theme/locales/en-US/ui.ftl) and [locales/zh-CN/ui.ftl](/Users/ysl/Code/theme/locales/zh-CN/ui.ftl).
- The Rust wrapper lives in [src/i18n/mod.rs](/Users/ysl/Code/theme/src/i18n/mod.rs).

## What Must Go Through i18n

- TUI menu bar, tab labels, panel titles, overlay titles, help text, config labels, footer hints, and status text
- GUI chrome text, button titles, section titles, config-sheet labels, and placeholders
- Editor-only preference labels and locale names

## What Must Not Be Localized Ad Hoc

- Theme token names
- Domain/source identifiers that are part of the theme model
- Export profile names and user-authored project names

If a future product decision requires translating domain-facing vocabulary, that should be added as an explicit domain-level mapping, not by inserting renderer-specific string rewrites.

## Implementation Constraints

- Do not hardcode user-visible UI copy in renderers when the string can live in i18n.
- Prefer typed translation keys in Rust over scattered string literals.
- Keep translation lookup in shared view/snapshot/update layers, not deep inside terminal/native rendering code, unless the text is purely native shell chrome.
- Locale selection is editor-only configuration and persists through [src/persistence/editor_config.rs](/Users/ysl/Code/theme/src/persistence/editor_config.rs).

## Verification Rule

- Every new UI translation key must exist in both locale files.
- The translation existence check is enforced by unit tests in [src/i18n/mod.rs](/Users/ysl/Code/theme/src/i18n/mod.rs).
- New UI-facing features should add their copy to Fluent first, then wire the Rust key, then extend tests if a new key family is introduced.
