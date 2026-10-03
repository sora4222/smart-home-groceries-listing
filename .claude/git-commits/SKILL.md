---
name: git-commits
description: Create focused Conventional Commits for changes in this repository. Use when staging or committing work; not for general Git recovery.
---

# Focused commits

- Commit each completed feature separately. Stage only its implementation,
  tests, docs, and relevant skill update; never use `git add .` for a
  multi-feature task.
- Use `feat`, `fix`, `refactor`, `test`, `docs`, or `chore` followed by a
  concise imperative description. Use `docs` for skill-only changes.
- Include a changed `Cargo.lock`; Docker builds with `cargo build --locked`.
- Before committing, inspect the staged diff and run the validation appropriate
  to the changed area. Do not commit unrelated existing changes.
