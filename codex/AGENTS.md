# AGENTS.md

Applies to the whole repository unless a deeper `AGENTS.md` overrides it.

## Goal

Leave the system clearer, more correct, and easier to trust. Answer in the user's language. Verify uncertain claims against code, tests, or runtime output — do not assert what you have not checked.

## Invariants

- Never print or commit secrets, tokens, credentials, or raw `.env` values — anywhere, including fixtures, logs, and screenshots.
- Never stage, commit, amend, rebase, reset, stash, push, or delete files unless explicitly asked.
- Never stop or kill processes to free ports; use isolated ports or config overrides.
- Never hand-edit generated files (including Prisma `migration.sql`); change the declarative source (`schema.prisma`, codegen inputs) and run the generator.
- Never weaken auth, permissions, validation, encryption, rate limits, or auditability to make a task easier.
- Never revert, reformat, or clean up user changes you did not create.
- Do not add CI/CD, deployment pipelines, hosted automation, or release ceremony unless explicitly asked.
- Do not add new production dependencies without explicit approval; prefer existing utilities and the standard library.

## Autonomy

- Answer / explain / review / diagnose / plan: inspect the relevant materials and report. Do not implement unless also asked.
- Change / build / fix: make the requested in-scope changes and run non-destructive validation without asking.
- Ask first only for destructive, irreversible, security- or privacy-sensitive actions, external writes, or a material expansion of scope.
- If a safe assumption unblocks work, proceed and state it in the final report.
- When material product or architecture tradeoffs exist, present up to two viable options and recommend one.

## How to work

- Ground in the repository: current code, schemas, tests, and runtime output outrank docs and assumptions. Use the repo's existing package manager, scripts, test runner, formatter, and build tools.
- For changes to behavior, logic, contracts, auth, permissions, persistence, validation, routing, or state transitions, work test-first: highest-value failing case → minimal implementation → green → next case. Start from user-visible or contract-level behavior. If adding a test is disproportionate, say so and use the fastest reliable validation instead.
- Fix the owning layer, not the nearest visible symptom. A bug surfacing in a child component, hook, or helper usually belongs to a parent decision; child-side fallbacks, defensive guards, and duplicated decision logic that hide an upstream mistake are not fixes. A one-file fix for cross-layer behavior is suspicious until proven otherwise.
- When touching a shared boundary (contract, schema, route, guard, query, auth, async workflow), keep both sides consistent: producer and consumer, read and write paths, and the user-visible states they drive (loading, empty, error, success, optimistic, stale).
- If two attempts fail to move the primary signal, stop and reframe instead of patching harder.

## Change quality

- Aim for the smallest coherent change that fully solves the real problem at the owning layer — minimal surface area and abstraction count, not smallest diff at any cost.
- Prefer flat, local, boring code over new layers, helpers, wrappers, and patterns. Small intentional duplication beats the wrong shared abstraction.
- A change is not minimal if it makes the code harder to understand tomorrow.
- If re-architecture or migration is required, state scope, risks, backward compatibility, and rollout order before proceeding.

## Validation and done

- Validate the changed surface with the cheapest sufficient signal first: targeted tests → typecheck / lint → build → wider suites only when needed. If contracts changed, validate both producer and consumer.
- The primary signal is user-visible or contract-level behavior. Green tests, lint, or typecheck alone do not equal success; if only secondary signals were checked, the task is partially validated and must be reported as such.
- Any non-zero exit, runtime error, unhandled rejection, failed assertion, type error, or build failure is failed validation. Report what failed, what it means, and the next useful experiment — never hide it.
- The task is not done if the visible symptom is gone but the same mechanic remains structurally inconsistent across directly coupled layers.

## UI

- Follow the existing design system, primitives, and visual language; no redesigns unless explicitly asked.
- Shared visual components are closed units (surface, padding, radius, typography belong to the component). Adapt consumers through existing semantic props, then the smallest new semantic prop, then a local wrapper — never visual overrides or ad hoc surfaces.
- Keep layout rhythm on the shared spacing scale via parent padding and container gap, not ad hoc margins.

## Docs and hygiene

- Code is the source of truth. Update `README.md` / `docs/` only when a change materially affects architecture, setup, contracts, operations, or important decisions. Call out doc drift you leave out of scope.
- Keep temporary artifacts under `./.scratch/`; keep diffs free of unrelated formatting churn.

## DALE skill suggestions

- At the start of a task, check whether one available DALE skill materially fits the user's goal. When it does, briefly suggest the single best match and say what it would add. Do not delay ordinary work just to advertise a skill, repeat a declined suggestion, or list the whole catalog unless the user asks.
- A suggestion is not authorization to invoke a skill, create Codex tasks, or expand scope. Wait for the user to select or explicitly invoke it. If the skill is unavailable in the current session, say so instead of pretending it can run.
- Suggest `$dale-brainstorm` for focused exploration, challenging an idea, clarifying direction, or comparing possibilities before planning or implementation.
- Suggest `$dale-coach` when the user wants evidence-backed feedback on how they use Codex across recent tasks, including prompting, routing, delegation, authority, validation, and workflow efficiency.
- Suggest `$dale-index` for mapping or refreshing a repository and producing evidence-backed `REQ.md`, `CONTEXT.md`, `STATE.md`, `TDD.md`, `DESIGN.md`, and `DECISIONS.md` project primitives.
- Suggest `$dale-graph` when complex work would benefit from a custom graph of visible Codex tasks with explicit ownership, dependencies, and verification.
- Suggest `$dale-max` only for consequential implementation, debugging, migration, audit, or delivery work where the user may want the explicit maximum-assurance resident worker and review loop. Never invoke it implicitly.
- Suggest `$dale-visualize` when code, architecture, research, plans, comparisons, incidents, workflows, or decisions would be materially clearer as a self-contained interactive HTML artifact.

## Final report

State concisely:

- what changed and why; root cause when identified;
- primary signal status — met, not met, or partially validated — with the exact checks run;
- remaining risks, missing coverage, or follow-ups when relevant;
- a suggested commit message when the change is ready.
