# Localization audit #258

Audited on 2026-11-17 against epic merge `80071bf7c5db2d88e957e4d2ffd9002b36a0cc06`
(#260, after SDK #259), plus the bounded fixes in this PR. Scope: #224, #225,
#226, #227. This is an independent source/resource/headless audit; no GUI was
launched and README was not edited.

**Disposition: the audit is delivered, but the epic is not yet fully localized.**
Passing resource contracts and headless regressions do not justify checking off
all of #224/#226/#227. Larger gaps below remain open; they are not silently
accepted deferrals or a claim that the epic is ready to merge into main.

## Requirement reconciliation

| Requirement | Evidence and disposition |
| --- | --- |
| #225 Locale, direction, fallback, Fluent, app overrides | Implemented in `crates/l10n/src/{locale,localizer,integration}.rs`; core and adapter suites exercise structural parents, Unicode extensions, malformed-resource atomicity, runtime switching and overrides. CLDR parent-locale negotiation is not implemented; structural fallback is documented. |
| #225 typed IDs and resource checking | `build_support` validates references, cycles, arguments and attributes; generated-consumer compile tests reject invalid calls. Lab build checks real and pseudo schemas. Explicit usage manifests are validated, but Lab's candidate manifest is not independent proof of actual UI use. |
| #225 ICU formatting and startup precedence | Number/percent/bytes/duration/date/time/list helpers are present, feature-gated and tested, including exact integers and numbering extensions. `LocaleRequest` supports CLI > setting > system > English. Lab wires CLI/system; it has no saved-locale persistence. Date/time helpers format caller-provided wall-clock values without timezone conversion. |
| #225 optional watch and API variations | No `fs-watch` feature exists. `L10nExt` is on `App`/deref contexts, not a standalone `Window` implementation. Generated functions replace the illustrative macro. No formatted-result cache. These are recorded implementation differences, not newly implemented capabilities or approved scope reductions. |
| #226 SDK snapshot and logical helpers | `infra/locale.rs`, `infra/locale/logical.rs`, adapter tests: direction, language, strings, numbers and refresh are published coherently. Explicit defaults/overrides have regressions. |
| #226 family direction/keyboard/geometry | Family tests exercise choice/groups, toolbar, selectors, collections, trees, tables, sidebar, menus, pager, stepper, slider/progress and overlays. Source review confirms physical arrows delegate through logical navigation; tree expansion swaps in RTL; table resize reverses delta and cancels on direction changes. This is strong sampled coverage, not a proof for every custom presenter. |
| #226 icons/looks/defaults | Opt-in mirrored artwork is separate from standard disclosure mirroring. Both look suites and shared conformance fixture pass. Shadcn listbox accessibility still contains English defaults (F4). Shared conformance covers two control types, not every control (F5). |
| #227 shell and pseudo-locales | SDK shell picker, startup locale, state-preserving render decorators, generated `en-XA`/`ar-XB`, and localization page are present. New audit probe uses shipped pseudo resources through an open selector, checking focus, selection, events, implicit defaults, explicit override and slot bounds. |
| #227 full UI adoption and formatting | Partial. The legacy registry/bridge leaves reachability, semantic arguments, plurals and direct-label gaps (F2/F3). Bounded bypass fixes are included, but no exhaustive full-app certification is made. |
| #227 first translations | All real locale schemas match. Expanded `ui-*` text is machine translated and still requires native-speaker review (F6). |
| #258 audit | This report, repeatable coverage JSON, independent checks and small fixes are delivered together. Open findings keep the parent completion criteria unsatisfied. |

## Findings fixed here

**F1 — RTL Parley alignment.** `apps/xgpui-lab/src/l10n/copy.rs` applied
`text_right()` to every RTL leaf. In xgpui-parley 0.8.0 `typeset.rs`,
`AlignKey::from(TextAlign::Right)` is logical **End**, while the default Left is
logical **Start**. Thus forcing Right undid natural RTL start alignment when
Parley was installed. The override is now native-fallback-only, consistent with
`screen/localized.rs`. A regression checks both installed backends.

**F2a — direct Workshop text bypasses.** Selection/feedback status strings,
choice state, collection state and fallback page/group/reason labels were passed
straight to GPUI text. These specific boundaries now use `l10n::ui` without
changing control entities or event/data schemas. This fixes lookup reachability;
it does not solve nested enum values or convert their sentences to semantic
Fluent messages (F3).

**F7 — destructive translation refresh drift.** `scripts/update_lab_copy.py`
added four spaces to already-indented Fluent continuation lines on every refresh.
It now preserves existing translated patterns verbatim. A temporary-catalog
regression verifies multiline preservation and repeat-run byte equality, and was
confirmed to fail against the pre-audit updater.

## Larger findings still open

**F2 — candidate extraction is not complete localization (high).** The Lab
registry contains static strings, format templates, developer diagnostics, code,
asset/license names and source identifiers. The usage manifest includes every
registry ID whether it reaches lookup or not. A zero-unused result therefore
means manifest consistency, not zero dead or missed UI strings. The extractor
scans only Lab Rust, skips attributes/tests/the l10n directory, does not expand
Rust macros and uses a capitalization/whitespace heuristic. For example,
`"{} colors"` from `src/color_stop_list/template.rs` is not registered because
its stripped literal is a lowercase single word. Data-role text deliberately
bypasses lookup. Remaining direct SDK renderers (for example badges in
`src/media_lab/resource_screen.rs`) need per-boundary classification; pipeline
identifiers must not be translated indiscriminately.

Follow-up: inventory actual display/accessibility boundaries, classify UI vs
code/user/sample data, replace in-scope UI with explicit typed messages, and
track genuine use independently from extraction. Do not bulk translate data to
make an unused counter green.

**F3 — reverse-format capture loses semantics (high).** There are 337 formatted
registry entries and eight duplicate literal-fragment groups. Different source
formats such as `Value: {:?}` and `Value: {:.0}` share a runtime shape; sorting
selects one constructor without knowing the originating call site. Formatting
specifiers and argument names are lost. Arguments are all strings, so the bridge
cannot choose grammatical numeric plural categories; only the dedicated demo
message currently contains a Fluent select. Nested UI states such as
`Dialog: open/closed` in `src/workshop/feedback.rs` remain source-language data
inside translated framing. Exact decimal capture works, but compound values,
percent/unit patterns, lists and date/time semantics are not generally recovered.

Follow-up: migrate these sentences to explicit typed IDs, pass raw numeric
selector values separately from localized display strings, and localize enum
labels separately from machine identifiers. Do not treat this as a separator
regex patch; it requires call-site migration and translation review.

**F4 — missed English accessibility defaults (medium).**
`crates/look-shadcn/src/controls/listbox_builder.rs` emits literal `Disabled` and
`Selection mode: {mode:?}` accessibility descriptions. They have no matching
`SdkStrings`/SDK resource hooks, so publishing an Arabic context cannot translate
them. This is UI accessibility copy, distinct from look usage metadata or explicit
app overrides. Follow-up needs a small SDK string-contract addition and both-look
accessibility assertions; this audit does not change the public string snapshot.

**F5 — conformance/clipping coverage is sampled (medium).**
`crates/sdk/src/infra/locale/conformance.rs` tests a checkbox and selector using
synthetic strings tagged as pseudo-locales. Both looks run it. Family tests add
many independent geometry/keyboard cases. The Lab layout test uses actual pseudo
resources at 720/1280 widths and two themes on native/Parley, but checks outer
slots and one row; it does not prove every glyph or every page avoids clipping.
The new actual-resource selector probe adds bounds and open-control preservation,
not whole-app visual certification. Custom-content override tests are present
in the SDK table/pager/selection suites; arbitrary user presenters remain app-owned.

Follow-up: broaden page/control-specific text-extent and accessibility checks,
then perform explicitly approved interactive/visual verification. No GUI launch
is implied by this audit or test success.

**F6 — translations are not linguistically approved (medium).** Expanded text
was locally machine translated with common-term corrections; provenance is in
`apps/xgpui-lab/localization/DEVELOPMENT.md`. Equality to English is reported, not
automatically classified as a defect: technical names, units, licenses and code
may legitimately be identical. Conversely changed text can still be wrong.
Native-speaker review, technical glossary review and argument/plural word-order
review remain outstanding. The user's later approval permits external machine
translation, but does not establish linguistic correctness of existing output.

## Resource evidence

`audit-258.json` is generated by `python3 scripts/audit_l10n.py`. It reports root
message entries and exact-text review candidates; the authoritative Fluent
syntax/reference/attribute/argument validation is `luma-l10n-check` and the build
checks, not this Python inventory reader.

| Catalog | English root messages | de / ja / ar missing or unknown | de / ja / ar identical to English |
| --- | ---: | --- | --- |
| Lab | 2,352 | 0 / 0 / 0 | 179 / 3 / 8 |
| SDK | 16 | 0 / 0 / 0 | 0 / 0 / 0 |

The Lab has 2,330 registry IDs, all included in the 2,352-ID usage manifest. No
manifest ID is absent from English and no English ID is absent from the manifest.
This is structural coverage only. The JSON retains exact equal-text IDs and all
eight duplicate capture groups for review. Generated pseudo schemas preserve
Fluent arguments/selectors; both real and pseudo resources load successfully.

## Text backend conclusions

Locale shaping options carry UI base direction and language; data/user text
uses `Auto`. Fluent leaves FSI/PDI around interpolated values. The independent
Arabic probe checks zero/one/two/few/many/other numeric categories using separate
Arabic-Indic display strings and asserts isolation. Another probe sends the actual Fluent-isolated Hebrew
filename through Parley: isolate marks add no visible advance and full selection
stays inside the layout. Existing paragraph API
regressions compare Parley's `Full` support with native `LeftToRight` support and
exercise natural/explicit paragraph edges.

Parley is the primary shaping path. Native fallback can align lines to the
requested edge but does not establish equivalent mixed-paragraph bidi reordering,
language-specific shaping/line breaking or single-line isolate rendering. The
localization page accurately labels the fallback when the typeset engine is not
installed. Headless options/string/layout tests do not certify every native
platform's font fallback or assistive-technology output.

## Physical geometry and scope exclusions

Reviewed remaining physical-edge examples include the Radix control-group focus
ring's symmetric left/right outset, slider thumb coordinates after logical value
mapping, table resize hit-stroke centering, and anchored popup/window coordinates.
Those uses should remain physical. A text search for `left`/`right` is not a bug
count. Artwork, chart/color axes and canvas positions remain physical.

TextField/TextArea editing (#105), docking, splitters/panels and titlebar geometry
are explicitly outside #226. Their continued physical behavior is not recorded
as an SDK mirroring regression. Lab shell placement is likewise not evidence
that all of the wider #224 application goal has been met.

## Verification

Commands and final results are recorded below after the audit fixes. All are
headless; no GUI, capture command, benchmark GUI or release application was run.

| Check | Result |
| --- | --- |
| `cargo fmt --all --check`; `git diff --check` | Pass |
| `cargo clippy --workspace --all-targets --offline -- -D warnings` | Pass |
| `cargo clippy --workspace --all-targets --features test-support --offline -- -D warnings` | Pass |
| `cargo test -p xgpui-luma -p xgpui-luma-look-radix -p xgpui-luma-look-shadcn -p xgpui-lab --features test-support --offline` | 1,964 tests/doctests pass; 32 existing ignores. SDK 883 unit, Radix 266 unit, Shadcn 351 unit, Lab 432 unit; 32 doctests pass. |
| Additional final `cargo test -p xgpui-lab --features test-support --offline l10n` after adding the isolate probe | 10 localization probes pass, including the additional Parley-isolate case. No product code changed after the full run. |
| `cargo test -p xgpui-luma-l10n --all-features --offline` | 47 tests/doctests pass |
| `cargo test -p xgpui-luma-l10n --no-default-features --features build-support,formatting --offline` | 34 tests/doctests pass |
| `cargo test -p xgpui-luma-l10n --no-default-features --offline` | 12 tests/doctests pass |
| Separate no-default `cargo check -p xgpui-luma-l10n --features <feature> --offline` for format-numbers, format-percent, format-duration, format-datetime, format-lists | All five pass |
| `luma-l10n-check` against English + de/ja/ar, for both Lab and SDK (Lab includes `--used`) | Pass; no schema differences |
| `cargo run -p lab-copy-catalog --offline -- /tmp/issue258-inventory.json`; `python3 scripts/update_lab_copy.py /tmp/issue258-inventory.json --check` | 2,330 registry entries verified |
| `python3 -m unittest discover -s scripts/tests -p 'test_update_lab_copy.py'` | Pass; the same test fails against the old updater |
| `python3 scripts/audit_l10n.py` | Produces the committed review report without modifying catalogs |

The existing `workflow_dispatch` localization workflow now also runs the updater
regression and prints translation review candidates. It remains opt-in. These
are local results; a hosted GitHub Actions run was not dispatched or claimed.

Resource reproduction (run from the repository root):

```sh
cargo run -p xgpui-luma-l10n --no-default-features --features build-support \
  --bin luma-l10n-check --offline -- \
  --reference apps/xgpui-lab/locales/en/lab.ftl \
  --locale de=apps/xgpui-lab/locales/de/lab.ftl \
  --locale ja=apps/xgpui-lab/locales/ja/lab.ftl \
  --locale ar=apps/xgpui-lab/locales/ar/lab.ftl \
  --used apps/xgpui-lab/locales/used-ids.txt
cargo run -p xgpui-luma-l10n --no-default-features --features build-support \
  --bin luma-l10n-check --offline -- \
  --reference crates/l10n/locales/en/sdk.ftl \
  --locale de=crates/l10n/locales/de/sdk.ftl \
  --locale ja=crates/l10n/locales/ja/sdk.ftl \
  --locale ar=crates/l10n/locales/ar/sdk.ftl
python3 scripts/audit_l10n.py > /tmp/l10n-audit.json
```
