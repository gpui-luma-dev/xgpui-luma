# Lab localization development

The shell selector and `--locale` accept `en`, `de`, `ja`, `ar`, `en-XA`, and
`ar-XB`; Unicode locale extensions continue to select number conventions.
The selector uses the existing SDK control. Locale changes refresh display
snapshots rather than replacing the shell, pages, editor values, or menu IDs.
No GUI is required for the resource and headless layout checks.

## Catalog maintenance

The legacy screen API accepts English strings. `src/l10n/copy.rs` resolves those
strings at presentation boundaries using generated, typed Fluent constructors.
`locales/copy.json` records stable source hashes, literal fragments, and Rust
source references. Common text roles, button presenters, selector/menu template
decorators, placeholders, tooltips, and explicit accessibility labels use this
bridge. New dedicated features should prefer explicit descriptive message IDs
as the localization demonstration does.

Dynamic UI copy retains `UiCopy` messages chosen at the call site. Rust formatting
specifications are applied to individual arguments before locale formatting;
`compose` distinguishes translated labels, numeric values, percentages, byte
counts, and raw data. Counts pass raw numeric selectors separately from localized
display digits. No runtime English pattern matching or argument capture remains.
An unknown English sentence stays unchanged and is a review candidate.

The inventory intentionally includes candidate technical strings and diagnostic
copy. It is not proof that every string reaches a translated UI boundary.
`TextKind::Data` keeps code, editable values, paths, sample text, and machine IDs
out of message lookup. Only exact numeric readouts receive numeric formatting.
Route names, benchmark report schemas, and control IDs remain source identifiers.

Run from the repository root:

```sh
cargo run -p lab-copy-catalog -- /tmp/lab-copy.json
python3 scripts/update_lab_copy.py /tmp/lab-copy.json
# Translate/review new locale entries, then check without writing:
python3 scripts/update_lab_copy.py /tmp/lab-copy.json --check
cargo test -p xgpui-lab --features test-support
```

The updater preserves translated messages by stable ID and leaves new ones in
English for review. `build.rs` checks locale ID/argument parity, used IDs, and
pseudo-locale schemas. The opt-in localization workflow also verifies the source
inventory. Source-only checks cannot prove semantic correctness or UI coverage;
the [independent audit #258](../../../docs/l10n/audit-258.md) is the historical
baseline. The [gap validation report](../../../docs/l10n/localization-gaps.md)
records this follow-up and its limits.

To capture independent render evidence without starting the application:

```sh
LAB_COPY_USAGE=/tmp/lab-copy-usage.json cargo test -p xgpui-lab --features test-support every_destination_survives
python3 scripts/audit_l10n.py --usage /tmp/lab-copy-usage.json > /tmp/lab-copy-audit.json
```

This visits every registered destination with native and Parley text, narrow/light
`en-XA` and wide/dark `ar-XB`. The report separates actual rendered IDs from
candidate definitions and lists unresolved text for review. It does not execute
every interaction, certify all leaf bounds, or replace visual review. The manual
localization workflow has a macOS job that publishes these observations.

## Translation provenance and review

Existing descriptive-ID translations are retained. The expanded `ui-*` catalogs
were generated **locally** on 2026-11-17, with manually corrected common shell
terms. No application strings were sent to a hosted translation service. These
expanded translations require native-speaker review, particularly technical
terminology and grammar around typed arguments. Equal IDs/arguments do not
establish translation quality.

Local models used:

- German: Argos `translate-en_de-1_3`.
- Japanese: Argos `translate-en_ja-1_1`.
- Arabic: Helsinki-NLP/OPUS-MT `opus-mt-tc-big-en-ar`, revision
  `ae9f1b2fd512000ca9440eb3b8ec9aa9acb298ac`.

[Argos Translate](https://github.com/argosopentech/argos-translate) provides the
local translation runtime and public model index. Arabic output was generated
using [OPUS-MT English–Arabic](https://huggingface.co/Helsinki-NLP/opus-mt-tc-big-en-ar),
by the Helsinki-NLP/OPUS-MT contributors, licensed CC BY 4.0; the output is a
machine-translated adaptation of this repository's English catalog. Model weights
and Python environments are not distributed with this repository.

Fluent arguments, escaped braces, and technical identifiers were protected during
translation. Both pseudo-locales transform Fluent text nodes while retaining
placeables/selectors; `en-XA` adds accents and approximately 40% padding, and
`ar-XB` also sets RTL direction. They are layout diagnostics, not translations.

## Backend limits

The localization page compares identical isolated text through a native GPUI
leaf and the `typeset` path. Without an installed Parley typeset engine, that
path falls back to native text and the page says so. Native headless layout does
not provide the same bidi shaping behavior as Parley. Logical row flow and SDK
locale direction are independent of backend shaping. Artwork, canvas coordinates,
color axes, and shell docking positions are not mirrored automatically.
