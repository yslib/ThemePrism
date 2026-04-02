# Export Template Engine Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current hardcoded template replacement exporter with a typed `ExportContext` + fixed-path template engine + filter pipeline, while keeping project export flow working end-to-end.

**Architecture:** Export remains profile-driven, but template rendering moves behind a small engine made of four focused parts: context building, template parsing, value evaluation, and filters. The plan preserves the current `event -> intent -> update -> effect -> export` flow and treats format-specific exporters as migration inputs, not the future model.

**Tech Stack:** Rust, existing domain/export models, `serde`, `thiserror`, `tempfile`, stdlib parsing/evaluation code, existing `cargo test` suite.

---

## File Structure

### Files to Create

- `src/export/context.rs`
  - Defines `ExportContext`, `ExportMeta`, `ExportValue`, and context construction from export inputs.
- `src/export/template/mod.rs`
  - Public template engine entry point; exposes `TemplateExporter` and small orchestration helpers.
- `src/export/template/parser.rs`
  - Parses template source into a minimal AST of text spans and placeholders.
- `src/export/template/eval.rs`
  - Resolves placeholder paths against `ExportContext` and applies filter chains.
- `src/export/template/filters.rs`
  - Implements built-in filters and type-checked filter application.

### Files to Modify

- `src/export/mod.rs`
  - Rewire export dispatch to use the new template engine and begin collapsing format-specific paths.
- `src/core/session.rs`
  - Pass richer export inputs if needed by `ExportContext` construction.
- `src/app/effect.rs`
  - If needed, expand `Effect::ExportTheme` payload so templates can read export metadata cleanly.
- `src/persistence/project_file.rs`
  - Keep project profile persistence aligned if `ExportFormat` changes shape.
- `src/app/update/project.rs`
  - Update default export profiles and any profile editing assumptions if format labels change.
- `src/app/view/helpers.rs`
  - Update export-profile summary labels if `ExportFormat` becomes template-only or template-first.
- `src/app/snapshot.rs`
  - Keep GUI config/export snapshot text aligned with the new export format model.

### Files to Remove or Shrink

- `src/export/template.rs`
  - Replace with the new module tree.
- `src/export/alacritty.rs`
  - Remove after migration, or convert to a bundled example template path during the compatibility task.

### Files to Test

- `src/export/template/mod.rs`
- `src/export/template/parser.rs`
- `src/export/template/eval.rs`
- `src/export/context.rs`
- `src/persistence/project_file.rs`
- `src/core/session.rs`

## Execution Rules

- Keep the current export UI untouched in the first tasks; only change it if the new export model forces a metadata rename.
- Use TDD for parser, evaluator, and context construction.
- Do not introduce a general template language.
- Do not expose arbitrary reflection over structs.
- Keep path keys aligned with stable enum `key()` values, not UI labels.

## Task 1: Add Typed Export Context

**Files:**
- Create: `src/export/context.rs`
- Modify: `src/export/mod.rs`
- Modify: `src/core/session.rs`
- Modify: `src/app/effect.rs`
- Test: `src/export/context.rs`

- [ ] **Step 1: Write failing context-construction tests**

Add tests that verify:
- `meta.project_name`, `meta.profile_name`, `meta.profile_format`, and `meta.output_path` exist
- every `TokenRole::key()` is present under `token`
- every `PaletteSlot::key()` is present under `palette`
- every `ParamKey::key()` is present under `param`

- [ ] **Step 2: Run the focused test target to verify failure**

Run: `cargo test export::context -- --nocapture`

Expected: FAIL because `src/export/context.rs` and its types do not exist yet.

- [ ] **Step 3: Implement `ExportContext` minimally**

Create:
- `ExportMeta`
- `ExportValue`
- `ExportContext`
- a builder that accepts the minimal export inputs needed from the existing export flow

Keep value types explicit:
- `Color`
- `Number`
- `Text`

- [ ] **Step 4: Thread export inputs into the context builder**

Modify the export call path so template rendering can build context from:
- project/profile metadata
- resolved token colors
- palette colors
- params

Only add data actually needed by the spec.

- [ ] **Step 5: Run the focused tests**

Run: `cargo test export::context -- --nocapture`

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/export/context.rs src/export/mod.rs src/core/session.rs src/app/effect.rs
git commit -m "feat: add typed export context"
```

## Task 2: Add Minimal Template Parser

**Files:**
- Create: `src/export/template/parser.rs`
- Modify: `src/export/template/mod.rs`
- Test: `src/export/template/parser.rs`

- [ ] **Step 1: Write failing parser tests**

Cover:
- plain text only
- one placeholder `{{token.comment}}`
- placeholder with whitespace
- placeholder with one filter
- placeholder with multiple filters
- malformed placeholder
- unclosed placeholder

- [ ] **Step 2: Run parser tests to verify failure**

Run: `cargo test export::template::parser -- --nocapture`

Expected: FAIL because parser types and functions do not exist yet.

- [ ] **Step 3: Implement the minimal AST and parser**

Define small types only:
- text segment
- placeholder segment
- parsed path
- filter list

Do not implement evaluation here.

- [ ] **Step 4: Run parser tests**

Run: `cargo test export::template::parser -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/export/template/mod.rs src/export/template/parser.rs
git commit -m "feat: add export template parser"
```

## Task 3: Add Filter Engine and Evaluator

**Files:**
- Create: `src/export/template/eval.rs`
- Create: `src/export/template/filters.rs`
- Modify: `src/export/template/mod.rs`
- Test: `src/export/template/eval.rs`
- Test: `src/export/template/filters.rs`

- [ ] **Step 1: Write failing evaluator tests**

Cover:
- `{{token.comment}}` default color rendering
- `{{token.comment | opaque_hex}}`
- `{{token.comment | rgba}}`
- `{{param.contrast | percent}}`
- `{{meta.profile_name | upper}}`
- unknown namespace/key error
- unknown filter error
- invalid filter-for-type error

- [ ] **Step 2: Run evaluator tests to verify failure**

Run: `cargo test export::template::eval -- --nocapture`

Expected: FAIL because evaluator and filters do not exist yet.

- [ ] **Step 3: Implement filter application**

Implement only the spec-approved first set:
- `hex`
- `opaque_hex`
- `rgb`
- `rgba`
- `alpha`
- `float`
- `percent`
- `lower`
- `upper`

Keep filters parameterless.

- [ ] **Step 4: Implement path lookup and evaluation**

Support only:
- `meta.<key>`
- `token.<key>`
- `palette.<key>`
- `param.<key>`

Reject any unsupported shape explicitly.

- [ ] **Step 5: Run evaluator tests**

Run: `cargo test export::template::eval -- --nocapture`

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/export/template/mod.rs src/export/template/eval.rs src/export/template/filters.rs
git commit -m "feat: add export template evaluation"
```

## Task 4: Replace Direct String Replacement Exporter

**Files:**
- Modify: `src/export/template/mod.rs`
- Delete: `src/export/template.rs`
- Modify: `src/export/mod.rs`
- Test: `src/export/template/mod.rs`

- [ ] **Step 1: Write failing integration tests for template export**

Cover:
- profile metadata expansion
- token placeholder expansion
- palette placeholder expansion
- param placeholder expansion
- mixed text + placeholder output

- [ ] **Step 2: Run the focused integration tests**

Run: `cargo test export::template -- --nocapture`

Expected: FAIL because old `string.replace()` implementation does not support the new path/filter behavior.

- [ ] **Step 3: Replace the old exporter internals**

Make `TemplateExporter`:
- read template text
- parse template once
- evaluate segments against `ExportContext`
- return precise `ExportError` values

- [ ] **Step 4: Remove the old flat replacement logic**

Delete:
- direct `rendered.replace(...)` path
- ad hoc “unknown token placeholder” scanning

- [ ] **Step 5: Run the focused integration tests**

Run: `cargo test export::template -- --nocapture`

Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add src/export/mod.rs src/export/template
git rm src/export/template.rs
git commit -m "refactor: replace template export with parsed engine"
```

## Task 5: Migrate Away from Built-In Format Exporters

**Files:**
- Modify: `src/export/mod.rs`
- Delete or shrink: `src/export/alacritty.rs`
- Modify: `src/app/update/project.rs`
- Modify: `src/persistence/project_file.rs`
- Modify: `src/app/view/helpers.rs`
- Modify: `src/app/snapshot.rs`
- Test: `src/persistence/project_file.rs`

- [ ] **Step 1: Decide the compatibility shape in code**

Keep one of these only if needed:
- temporary compatibility branch for `ExportFormat::Alacritty`
- or direct migration to template-backed profiles

Do not keep `AlacrittyExporter` as the long-term design.

- [ ] **Step 2: Write failing migration/persistence tests**

Cover:
- old project file with `alacritty` format still loads
- default export profiles remain usable
- template profile round-trips through project persistence

- [ ] **Step 3: Run the migration tests to verify failure**

Run: `cargo test persistence::project_file -- --nocapture`

Expected: FAIL for whichever compatibility or migration path has not yet been implemented.

- [ ] **Step 4: Implement compatibility or migration**

Preferred direction:
- make template export the primary model
- represent bundled examples as template paths, not dedicated exporters

If a compatibility branch is needed, isolate it and document it as transitional.

- [ ] **Step 5: Update default export profiles**

Adjust:
- default names
- default output paths
- bundled template paths

Keep profile UI assumptions intact unless names visibly change.

- [ ] **Step 6: Run migration tests**

Run: `cargo test persistence::project_file -- --nocapture`

Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add src/export/mod.rs src/app/update/project.rs src/persistence/project_file.rs src/app/view/helpers.rs src/app/snapshot.rs
git commit -m "refactor: make template export the primary export format"
```

## Task 6: End-to-End Export Verification

**Files:**
- Modify: tests only as needed in touched files
- Optionally add: `templates/` bundled example templates if required by the migration path

- [ ] **Step 1: Add end-to-end tests through the export flow**

Cover:
- enabled profile export produces output
- exported artifact path is correct
- a template using `meta`, `token`, `palette`, and `param` all renders correctly

- [ ] **Step 2: Run targeted end-to-end tests**

Run: `cargo test export -- --nocapture`

Expected: PASS

- [ ] **Step 3: Run full verification**

Run:
- `cargo fmt`
- `env -u http_proxy -u https_proxy cargo check`
- `env -u http_proxy -u https_proxy cargo test`

Expected:
- formatting succeeds
- compile succeeds
- all tests pass

- [ ] **Step 4: Commit**

```bash
git add src/export src/core/session.rs src/app/effect.rs src/persistence/project_file.rs src/app/update/project.rs src/app/view/helpers.rs src/app/snapshot.rs templates
git commit -m "test: verify end-to-end template export flow"
```

## Task 7: Clean Up Temporary Compatibility and Document the Stable Contract

**Files:**
- Modify: `src/export/mod.rs`
- Modify: `docs/superpowers/specs/2026-04-02-export-template-engine-design.md` only if implementation requires a documented clarification
- Modify: root-level user-facing docs only if export behavior visibly changes

- [ ] **Step 1: Remove temporary compatibility code if it is no longer needed**

Delete transitional branches that were only required for migration and are not part of the desired final design.

- [ ] **Step 2: Add or update tests that prove stable export keys come from enum `key()` values**

Cover:
- token keys
- palette keys
- param keys

- [ ] **Step 3: Run focused cleanup tests**

Run: `cargo test export::context -- --nocapture`

Expected: PASS

- [ ] **Step 4: Run final verification again**

Run:
- `cargo fmt`
- `env -u http_proxy -u https_proxy cargo check`
- `env -u http_proxy -u https_proxy cargo test`

Expected:
- all green

- [ ] **Step 5: Commit**

```bash
git add src/export docs/superpowers/specs/2026-04-02-export-template-engine-design.md
git commit -m "chore: finalize stable export template contract"
```

## Notes for Implementers

- Keep parser, evaluator, and filter code separate. Do not collapse them into one file “for convenience”.
- Do not expose UI labels or localized strings as export keys.
- Do not add filter parameters.
- Do not add loops or conditionals as “just one small extension”.
- Keep errors precise and user-facing.
- Prefer bundled example templates over built-in format exporters.
