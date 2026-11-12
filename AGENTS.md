# AI coding instructions

Always answer concisely. Prefer short bullets. Do not give long explanations unless I ask.

Documentation:
- DO NOT EDIT `README.md` without explicit user approval. It is an independently maintained document and should generally not be touched by agents.

Git workflow and branching (Epic Branch Pattern):
- Never commit directly to `main` or `develop`.
- Never create worktrees under `/private`, including `/private/tmp`. Place worktrees beside the repository in its parent directory, for example `/Users/scg/Developer/GitHub/xgpui-luma-issue-154`.
- When assigned an issue (`<ISSUE_ID>`), isolate work under `issue-<ISSUE_ID>/epic`.
- If `origin/issue-<ISSUE_ID>/epic` does not exist, branch it from `origin/main` and push it.
- Branch subtasks from the epic branch using: `issue-<ISSUE_ID>/<short-task-slug>`.
- Set the PR base target to `issue-<ISSUE_ID>/epic` (never target `main` directly).

Pull requests and git:
- You are free to create, push to, and update PRs targeting `issue-<ISSUE_ID>/epic` without prior approval.
- Do not merge PRs into `main` or close PRs without explicit user approval.
- Prefer small, reviewable diffs.

Cleanup and handoff:
- After merging a PR, verify the task branch is merged, delete it locally and remotely, and remove its clean worktree.
- Keep an epic branch until it is merged into `main`, then delete it locally and remotely and remove its clean worktree.
- Never delete branches with unmerged work or dirty worktrees without explicit user approval.
- Report any branches or worktrees left behind and explain why they remain.
- State the current checkout's branch in the final handoff.

GUI applications:
- Do not launch or run GUI applications without explicit user approval to launch them.
- Approval to implement changes, build, or test does not authorize GUI launches, including startup or smoke tests.

Before making changes:
- Read the relevant source modules and repository instructions directly; do not rely only on summaries.
- Inspect relevant Rust modules directly; do not rely only on summaries.
- In apps under `apps/`, use SDK controls and theme factories (`ShadcnLook`, `*Builder`, `.spawn(cx)`). Do not home-brew buttons, checkboxes, inputs, or other interactive chrome with raw `div` styling.
- Run `cargo fmt`, `cargo clippy`, and relevant tests before finalizing when possible.

Rust conventions:
- Preserve existing error handling style.
- Do not introduce panics in library code unless existing conventions allow it.
- Prefer explicit types where they improve readability.
- Keep public API changes minimal and documented.
