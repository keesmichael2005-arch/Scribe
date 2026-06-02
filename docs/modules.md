# Scribe — Modules

## What a module is

Modules are coherent business or technical surfaces. Each module owns its internal state and complexity behind a narrow public interface. Every module lives in its own directory directly under `src/<module>/` with an `index.ts` barrel that exports the public API. Other code only imports through the barrel. There is no `src/lib/` layer — modules sit directly under `src/`.

Sizing is by domain, not line count. A module is big enough when its public interface is narrow relative to its internal surface. Single-file "modules" and utility grab-bags aren't modules. Fold them into their owner.

## Module table

| Module | Purpose | Public interface | Depends on |
|---|---|---|---|
| _seeded empty, first sprint to add a module populates the table_ |  |  |  |

## Dependency rules

- Cross-module access goes through `<other-module>/index.ts` only. Reaching into a sibling module's internals is a boundary violation.
- A module's internal files may not be imported from outside the module.
- Cycles between modules are forbidden. If A and B need each other, the shared surface is its own module.
- `src/app/` route files are entry shells only, ≤10 lines each, delegating into modules. No business logic in route files.

## When to add a module

New modules are declared in the sprint PRD or sprint design note with rationale and proposed public interface. Don't create modules silently inside tasks. After sprint ship, OHQ appends new modules from the PRD into this file automatically.
