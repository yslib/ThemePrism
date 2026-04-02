# Export Template Engine Design

## Summary

The current export layer has two shapes:

- a dedicated `AlacrittyExporter`
- a minimal `TemplateExporter` that performs direct `string.replace()` calls

That is enough to prove the export path works, but it is not the right long-term product model. The product goal is not to embed one exporter per app. The product goal is to let users describe output formats through templates while the application supplies a stable, typed theme context.

This spec replaces format-specific exporter growth with a single engineering direction:

- one general template-based export system
- one stable `ExportContext`
- fixed placeholder paths
- a small filter pipeline
- no control flow language
- no format-specific exporter logic

The system should stay aligned with the current architecture and refactoring principles:

- single information source
- data-driven boundaries
- small focused components
- no hidden business logic in UI or render code

## Goals

- Make template export the primary export mechanism.
- Remove the need to add built-in exporters for individual apps or tools.
- Expose a stable export data model aligned with the app/domain model.
- Support fixed placeholder paths for:
  - `meta.*`
  - `token.*`
  - `palette.*`
  - `param.*`
- Support a small filter chain for formatting values.
- Keep the template engine constrained enough that it does not turn into a scripting language.
- Make future theme model growth flow naturally into export support.

## Non-Goals

- This spec does not add arbitrary expressions.
- This spec does not add `if`, `for`, `let`, or loops.
- This spec does not add user-defined functions or filters.
- This spec does not add dynamic path lookup or reflection over arbitrary structs.
- This spec does not add per-format built-in exporters beyond optional bundled template examples.
- This spec does not redesign project/export UI in the same pass.

## Current Problem

The current template exporter is intentionally small, but it now has the wrong shape for product growth.

Problems:

1. Placeholder handling is hardcoded in one file.
2. The current implementation only supports a narrow subset of output needs.
3. There is no typed value model, so formatting behavior does not scale.
4. Export-facing keys are not yet defined as an explicit stable interface.
5. The presence of `AlacrittyExporter` pulls the design toward built-in format support instead of a general export engine.

If left as-is, the system will grow by accretion:

- one more placeholder
- one more custom formatting branch
- one more dedicated exporter

That is exactly the kind of growth this codebase has been moving away from.

## Approach Options

### Option 1: Keep expanding direct placeholder replacement

Continue extending the current `TemplateExporter` with more hardcoded replacements.

Pros:

- fastest short-term path

Cons:

- weak engineering boundary
- poor extensibility
- higher risk of duplicated logic
- weak error reporting
- encourages format-specific special cases

This option is rejected.

### Option 2: Fixed-path template engine with typed context and filters

Introduce:

- a stable `ExportContext`
- a tiny placeholder parser
- a path evaluator
- a filter pipeline

Template syntax stays deliberately small:

- `{{token.comment}}`
- `{{token.comment | opaque_hex}}`
- `{{palette.accent_2 | rgb}}`

Pros:

- general enough for most theme formats
- constrained enough to stay maintainable
- aligned with current engineering principles
- no format-specific exporter logic required

Cons:

- requires building a small parser/evaluator subsystem

This option is recommended.

### Option 3: Use a full general-purpose template engine

Adopt something like `tera`, `handlebars`, or `liquid` and expose a data tree.

Pros:

- maximum flexibility

Cons:

- too much capability for the current need
- harder to constrain and document
- greater risk of turning export into a second programming environment
- likely to create a wider long-term support surface than the product needs

This option is deferred.

## Recommended Design

### 1. `ExportContext` is the stable export boundary

Templates should not read directly from `AppState`, `ResolvedTheme`, or arbitrary domain structs.

Instead, export first builds a stable read model:

- `ExportContext`

This context becomes the single data source for template evaluation.

Its purpose is:

- provide export-safe values
- define stable path names
- isolate templates from internal state layout
- make export behavior testable without the full app runtime

### 2. Export namespaces mirror the product model

The template engine should expose these top-level namespaces:

- `meta`
- `token`
- `palette`
- `param`

Those names are stable and explicit.

The contents should map closely to existing model enums and keys:

- `token.comment` should align with `TokenRole::Comment`
- `palette.accent_0` should align with `PaletteSlot::Accent0`
- `param.contrast` should align with `ParamKey::Contrast`

The export engine should not invent a second naming scheme for these concepts.

This is a critical rule:

**UI labels are not export identifiers. Stable `key()` values are export identifiers.**

### 3. Fixed path syntax only

The template engine should support only fixed path lookups:

- `{{meta.project_name}}`
- `{{token.comment}}`
- `{{palette.accent_3}}`
- `{{param.contrast}}`

The first implementation should not support:

- nested arbitrary object traversal
- indexing
- computed paths
- dynamic key lookups

This keeps both parsing and error messages simple and predictable.

### 4. Filter chains are the only transformation mechanism

Templates need formatting flexibility, but not a mini-language.

The supported transformation model should be:

- base value lookup
- zero or more filters applied left to right

Examples:

- `{{token.comment | hex}}`
- `{{token.comment | opaque_hex}}`
- `{{token.comment | rgba}}`
- `{{param.contrast | percent}}`
- `{{meta.profile_name | upper}}`

This is enough to cover most real theme-file formats without introducing general logic.

### 5. No control flow

The first engine must not support:

- `if`
- `for`
- `match`
- arithmetic
- temporary variables
- macros

If a target format eventually requires that class of logic, it should be treated as an explicit future design decision, not something silently grown from this engine.

## Data Model

### `ExportContext`

The export context should be an explicit struct with these sections:

- `meta`
- `token`
- `palette`
- `param`

Recommended shape:

- `ExportContext`
- `ExportMeta`
- internal typed value table or namespace table

The exact storage can be implemented however is most ergonomic, but the public conceptual model should stay stable.

### `ExportMeta`

This namespace should contain export-session information, not theme model data.

Recommended first fields:

- `project_name`
- `profile_name`
- `profile_format`
- `output_path`

These values are useful in many templates and do not belong in token/palette/param namespaces.

### `ExportValue`

Template evaluation should operate on typed values, not only strings.

Recommended types:

- `Color`
- `Number`
- `Text`

This allows filters to enforce type rules and produce precise errors.

The engine should not flatten everything to strings before filter evaluation.

### Stable keys

The following should be treated as stable external export keys:

- `TokenRole::key()`
- `PaletteSlot::key()`
- `ParamKey::key()`

Changing those names in the future should be considered a breaking export-interface change.

## Template Syntax

### Placeholder form

Only one placeholder form is required:

- `{{ path }}`
- `{{ path | filter }}`
- `{{ path | filter1 | filter2 }}`

Whitespace around the path and filter separators can be tolerated, but the grammar should stay minimal.

### Path form

The supported grammar should be equivalent to:

- `<namespace>.<key>`

Where:

- namespace is one of `meta`, `token`, `palette`, `param`
- key is a stable exported key for that namespace

This avoids the complexity of arbitrary deep traversal.

## Filter Design

### First filter set

The first filter set should be small and high value.

Color filters:

- `hex`
- `opaque_hex`
- `rgb`
- `rgba`
- `alpha`

Number filters:

- `float`
- `percent`

Text filters:

- `lower`
- `upper`

### Default rendering

If no filter is provided:

- `Color` defaults to canonical hex output
- `Number` defaults to a compact numeric string
- `Text` defaults to the original string

This reduces template noise while keeping filters available when formatting matters.

### Type checking

Filters must be type-aware.

Examples of invalid uses:

- `{{token.comment | percent}}`
- `{{param.contrast | hex}}`

These should return explicit export errors rather than silently coercing values.

## Error Handling

The new export engine should produce precise, user-facing template errors.

The engine should distinguish at least:

- invalid placeholder syntax
- unclosed placeholder
- unknown namespace
- unknown path key
- unknown filter
- invalid filter for value type
- missing context value

Errors should include enough context to debug templates quickly:

- offending snippet when possible
- the path or filter name
- approximate source position

This is a major quality improvement over raw string replacement failure modes.

## Architecture

Recommended module split:

- `src/export/mod.rs`
  - export profiles
  - export format dispatch
- `src/export/context.rs`
  - `ExportContext`
  - conversion from app/domain export inputs
- `src/export/template/mod.rs`
  - public template export interface
- `src/export/template/parser.rs`
  - placeholder parsing into a small AST
- `src/export/template/eval.rs`
  - path lookup and filter evaluation
- `src/export/template/filters.rs`
  - filter implementations

This keeps parsing, context construction, and evaluation separate.

## Export Format Model

### Long-term direction

The long-term direction should be:

- one primary export mechanism: templates

That means the special-case `Alacritty` exporter should not continue as the model for future formats.

### Practical transition

To avoid breaking existing flows abruptly, the implementation can transition in two steps:

1. introduce the new template engine
2. convert built-in exporter examples such as Alacritty into bundled template files

After that, `ExportFormat` should ideally collapse toward a template-only model.

If a compatibility window is needed, it should be treated as temporary migration support, not as the future architecture.

## Testing Strategy

The new export engine should be tested at three layers.

### 1. Context construction tests

Verify that:

- every token key is present
- every palette key is present
- every param key is present
- meta values are populated correctly

### 2. Parser/evaluator tests

Verify:

- plain path lookup
- multiple filters
- default rendering
- unknown path errors
- unknown filter errors
- type mismatch errors
- malformed placeholder errors

### 3. Integration tests

Verify end-to-end export with:

- a bundled template example
- a profile-driven export path
- representative token/palette/param placeholders

## Migration Guidance

The current `TemplateExporter` should be treated as replaceable implementation, not as a stable API.

Migration should preserve:

- project file structure for export profiles where practical
- enabled/disabled export target behavior
- output path handling

But implementation should move away from direct string replacement toward the typed engine.

## Open Decisions Explicitly Deferred

These questions are intentionally out of scope for the first pass:

- filter parameters
- conditionals
- loops
- user-defined aliases inside templates
- user-defined custom filters
- multi-provider export contexts
- GUI template editor UX

Those should only be revisited if real export use cases prove the limited engine is insufficient.

## Recommendation

Build a small, typed template engine around a stable `ExportContext` and fixed path namespaces.

This gives the product:

- a general export mechanism
- a stable external template contract
- enough formatting power for real theme file generation
- none of the complexity of a general scripting language

That is the best fit for the current product direction and the codebase’s engineering constraints.
