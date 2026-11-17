# Localization gaps follow-up

Baseline: #261, epic commit `ce513f713d3eff9a354f6bf001236af7e6d8dbb0`.
The historical [audit #258](audit-258.md) remains unchanged.

## Changes

- **F2:** Lowercase UI boundaries and expression-list macros join the source
  inventory. Explicit typed references survive removal of English format literals.
  Registry entries identify typed references and UI-boundary candidates. A separate
  headless observer records actual rendered IDs and unresolved text; the generated
  resource manifest is explicitly a contract, not proof of UI reachability.
- **F3:** Removed English reverse-format matching, including ambiguous fragment
  dispatch. Dynamic captions, state readouts, benchmark statuses and findings now
  choose generated constructors at call sites. Retained messages resolve on locale
  changes. Raw identifiers/sample data stay separate from translated copy.
  Semantic state keys, numeric plural selectors, formatted display digits, ICU
  percentages, byte counts and lists replace rendered-sentence inference.
- **F4:** Shadcn listbox disabled and selection-mode descriptions use live SDK
  strings. Defaults preserve compatibility; locale resources and application
  overrides have adapter and accessibility-node regressions.
- **F5:** Every registered Lab destination renders headlessly under native and
  Parley text, narrow/light expanded pseudo text and wide/dark RTL pseudo text.
  Workspace bounds and route retention are checked. Existing focused tests cover
  open popups, focus, selection/events, explicit overrides and bidi isolates.
  The opt-in workflow adds the complete SDK/look/Lab suites and coverage artifacts.
- **F6:** New source messages have German, Japanese and Arabic drafts generated
  locally using the models attributed in the Lab development guide. Semantic
  messages were authored as drafts. Translation completeness is separate from
  linguistic approval; native-speaker review remains pending.

## Reproducible evidence

See [Lab development commands](../../apps/xgpui-lab/localization/DEVELOPMENT.md).
`audit_l10n.py --usage <capture>` reports actual exercised IDs separately from
unexercised definitions. Unknown strings remain review candidates: numeric ranges,
protocol codes, font/sample names and identifiers are intentionally present in the
Lab. Source candidates and same-as-English entries are not automatically defects.
The JSON snapshot alongside this report records the tested revision's observation.

## Review limits

No GUI application was launched. Headless route visits cover default states and
workspace containment, not every interaction or visual leaf. Physical device
color output, platform dialogs, fonts, text clipping and native-backend bidi still
need manual inspection. Machine translation and pseudo expansion cannot certify
language quality. These limits remain explicit review work; this follow-up does
not assert that every acceptance criterion in #224 is certified or authorize an
epic merge into main.
