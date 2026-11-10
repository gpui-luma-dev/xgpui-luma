# Launching a page and taking snapshots

> The docs site's *XGPUI Lab App* page (`docs/xgpui-lab.html`) repeats these
> options. Update it when you add or change one, or it drifts.

## Start the Lab on a page

```bash
cargo run -p xgpui-lab -- --open "xgpui-media/Image Filters"
cargo run -p xgpui-lab -- --open Typography --theme light
cargo run -p xgpui-lab -- --open "Shell Lab" --hide-nav
cargo run -p xgpui-lab -- --list-pages
```

- `--open PAGE` starts on that page and expands its section in the sidebar.
  `PAGE` is `section/page` as `--list-pages` prints it, or a page name alone when
  it is unique (`Typography`). Matching ignores case. An ambiguous name
  (`Gamut Mapping`) lists the full names to choose from.
- `--theme light|dark` starts in that theme.
- `--hide-nav` starts with the navigation panel closed; the titlebar toggle
  reopens it at its normal width.
- `--size WIDTHxHEIGHT` sets the window size (at least 720x480; default 1440x880).

## Capture one page

```bash
cargo run -p xgpui-lab -- --open "Color Spaces" --size 1440x900 --capture /tmp/color-spaces.png
```

`--capture PNG` opens the normal Lab window on the page, waits for it to settle
(`--settle MS`, default 3000), saves the window's rendered frame and quits. It
captures exactly what you see, including the titlebar and sidebar.

## Regression snapshots

[`scripts/lab_snapshots.py`](../scripts/lab_snapshots.py) builds once and captures
pages in dark and light, then compares two runs:

```bash
python3 apps/xgpui-lab/scripts/lab_snapshots.py capture target/lab-snapshots/before
# change code
python3 apps/xgpui-lab/scripts/lab_snapshots.py capture target/lab-snapshots/after
python3 apps/xgpui-lab/scripts/lab_snapshots.py compare target/lab-snapshots/before target/lab-snapshots/after
```

- By default it captures the `LabScreen` pages. Use `--page` (repeatable),
  `--theme` and `--size` for others. `--hide-nav` captures the page without
  the navigation panel.
- `compare` reports `same`, `CHANGED` and `MISSING`, and exits non-zero when
  anything changed. Byte-identical PNGs are identical renders. With ImageMagick
  installed it also counts differing pixels and writes highlighted diffs to
  `after/diff/`.
- Keep the size and theme the same between runs. Animated pages (waves, video,
  sweeps) differ from run to run; leave them out or expect changes.
- Each capture opens a native window, so run it on a machine where that is fine.
