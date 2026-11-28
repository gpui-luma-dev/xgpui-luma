\# Project Rules \& Rust Workflow



\## Fast Edit-Check Loop (IMPORTANT)

\- During iterative edits and refactoring, ONLY run `cargo check --tests --message-format=short` to verify compilation and borrows.

\- NEVER run `cargo build` or bare `cargo test` across the full workspace during intermediate steps—this repo has a large test suite and full builds stall iteration.

\- Run targeted tests only for the module or binary you touched:

&#x20; - Unit tests: `cargo test --lib <module\_name> -- --nocapture`

&#x20; - Integration test: `cargo test --test <test\_file\_name>`

\- Skip doctests (`--lib --bins --tests`) unless public doc comments were modified.



\## Rust Refactoring Guardrails

\- Keep changes tightly scoped to the requested refactor. Do not opportunistically rewrite unrelated modules or extract unsolicited traits.

\- Do NOT introduce `.clone()`, `Arc`/`Rc`, `Box::leak`, `'static` bounds, or `unsafe` just to silence borrow-checker errors without explaining why ownership cannot be restructured cleanly.

\- Before reporting a multi-file refactor complete, run `cargo clippy --workspace --all-targets -- -D warnings` (or scoped `-p <crate>`) and `cargo fmt --all -- --check`.



\## Test Hygiene

\- When modifying existing behavior, update or consolidate redundant tests rather than duplicating copy-pasted test functions.

\- Prefer table-driven loop assertions inside a single `#\[test]` when testing multiple input/output cases for the same function.



\## Communication \& Tool Efficiency

\- Be concise: batch file reads and independent edits in parallel tool calls rather than reading one file per turn.

\- State the plan in 2–3 bullets before touching more than 3 files.

