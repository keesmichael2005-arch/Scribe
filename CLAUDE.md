# CLAUDE.md — Scribe

This file is the stable orientation for any Claude Code session working in this repo. It is hand-maintained and append-only. OHQ does not overwrite it. New conventions, gotchas, and non-negotiable rules get added here over time, proposed by sprint retros and approved before they land.

## What this project is

_(One or two sentences. Update as the project matures.)_

## Module discipline

This codebase follows module discipline. Read `docs/modules.md` before editing any code.

- Every module lives in its own directory directly under `src/<module>/` with an `index.ts` barrel that exports the public API. There is no `src/lib/` layer.
- Cross-module access goes through `<other-module>/index.ts` only. Reaching into a sibling module's internals is a boundary violation.
- New modules are declared in the sprint PRD or sprint design note up front, never created silently inside a task.

## Codebase navigation

_(Key entry points, how to run, how to test. Update as the codebase grows.)_

## Pointers

- `docs/modules.md` — canonical Module Map.
- `docs/architecture.md` — current blueprint's architecture content (regenerates at blueprint approval).
- `docs/blueprint.md` — current blueprint (regenerates at blueprint approval).
- `documentation/02-prds/` — per-sprint PRDs.
- `documentation/01-build-order.md` — current sprint sequence.
