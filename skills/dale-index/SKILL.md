---
name: dale-index
description: Index or refresh a software repository through four visible Codex tasks and produce evidence-backed REQ.md, CONTEXT.md, STATE.md, TDD.md, DESIGN.md, and DECISIONS.md primitives. Use when the user invokes $dale-index, selects Dale Index through /skills, asks to index or map a repository, bootstrap project primitives, recover project context, or update stale primitive documents after substantial repository changes.
---

# Dale Index

Build a compact source of truth from the repository as it exists now. Use three
parallel discovery tasks, one adversarial verification task, and the calling task
as the sole integrator.

## Keep the orchestration thread-native

- Treat explicit `$dale-index` selection, a direct request to run Dale Index, or
  an explicit request for multiple Codex tasks as authorization to create the
  visible tasks described here.
- If this skill matched implicitly from an ordinary documentation request, tell
  the user that Dale Index creates four visible tasks and get consent before
  calling `create_thread`.
- Use `list_projects`, `create_thread`, `set_thread_title`, `wait_threads`,
  `read_thread`, and `send_message_to_thread`.
- Never use subagents, `spawn_agent`, or hidden delegation as a substitute.
- Make every created task user-owned and inspectable. Do not archive it unless
  the user asks.
- Explicit `$dale-index` selection or a direct request to run Dale Index also
  delegates per-task Luna and Astra model and reasoning selection to this skill. For an
  implicit match, get consent for both visible task creation and automatic
  routing; otherwise omit `model` and `thinking`.
- Read `references/model-routing.md` before dispatch. Explicit Luna or Astra model or
  reasoning constraints in the current request override automatic routing. If
  a requested route falls outside the Luna and Astra boundary or the live tool schema,
  state the conflict instead of silently substituting another model.
- In every worker prompt, forbid that worker from creating subagents or more
  tasks.
- If the Codex task tools are unavailable or the repository cannot be resolved
  to one saved project, stop and report the exact blocker. Do not silently run a
  single-task imitation.

## Produce the primitive set

Use these files as distinct, non-overlapping sources of truth:

| File | Owns |
|---|---|
| `REQ.md` | Product intent, users, scope, requirements, acceptance criteria |
| `CONTEXT.md` | Architecture, repository map, contracts, constraints, operations |
| `STATE.md` | Current objective, active work, blockers, risks, next actions |
| `TDD.md` | Test strategy, commands, quality gates, missing coverage |
| `DESIGN.md` | Existing visual language, tokens, primitives, states, accessibility |
| `DECISIONS.md` | Dated decisions, alternatives, evidence, consequences, status |

Keep `STATE.md` volatile and `DECISIONS.md` durable. Do not duplicate a fact
across documents; link to its owning file. If the project has no UI, say so with
evidence in `DESIGN.md` instead of inventing a design system.

Place missing primitives in the repository root, matching the reference layout.
If an existing complete primitive set lives together elsewhere, update it in
place. Never create a second competing set.

Use the schemas under `assets/primitives/` when creating files. Treat them as
structure, not prose to copy blindly. Remove every placeholder from files
written to the target repository.

## Run the indexing workflow

### 1. Establish the evidence boundary

1. Resolve the repository root.
2. Read every applicable `AGENTS.md`, including deeper files for scoped paths.
3. Inspect `git status`, tracked files, manifests, lockfiles, runtime entrypoints,
   tests, schemas, routes, configuration examples, and existing documentation.
4. Record the current branch or revision when Git exists. Never mutate Git state.
5. Label repository evidence as `verified`, `inferred`, `unknown`, or `stale`.
6. Treat code, tests, schemas, and runtime output as stronger evidence than docs.

Do not print secrets or raw environment values. Use names of required variables,
not their values.

### 2. Resolve the Codex project

Call `list_projects` first. Select the one exact saved project whose root matches
the repository. Use `target.environment.type: "local"` because discovery tasks
are read-only and the calling task owns all writes. If selection is ambiguous,
ask for the missing project choice before creating tasks.

### 3. Launch three discovery tasks

Read `references/index-roles.md` and `references/model-routing.md` before
creating tasks. Immediately before dispatch, inspect the current `create_thread`
tool schema and intersect its supported combinations with the Luna and Astra catalog.
Route each role from its actual repository scope and evidence risk; do not copy
the coordinator's model settings. Show each role's model, thinking, and one-line
routing reason, then create these roles concurrently:

1. `Dale Index · Context Cartographer`
2. `Dale Index · Product & State Historian`
3. `Dale Index · Quality & Design Auditor`

Give each task the shared evidence contract and its role contract. Require
read-only inspection and a structured final report; do not let discovery tasks
edit primitives.

Create every role in the selected project's `local` environment. Before
creation, publish a coordinator manifest with title, exact `projectId`, expected
host, role, and `dispatching` status. Immediately record the exact returned
`threadId`, `hostId`, status, and wait cursor and preserve this registry in
continuation summaries. Use known IDs directly for read, send, and wait;
`list_threads` is optional discovery only and cannot gate a known ID or prove
absence. Validate a user-supplied actual ID with `read_thread` before adoption.
Never use a `clientThreadId` as a real ID; an unexpected client ID or ambiguous
creation becomes uncertain and must never be recreated.

Rename each task immediately with `set_thread_title`. Retain its model,
thinking, and routing reason in the same manifest.

### 4. Collect without busy polling

Wait on all active tasks with one bounded `wait_threads` call. Reuse
`afterCursor` values so completed text is not replayed. A progress timeout is
normal; report meaningful changes only. Use `read_thread` only when the compact
result lacks evidence needed for integration.

If a report violates its contract, send one precise correction with
`send_message_to_thread` and wait again. Preserve the task's current route
unless the required correction materially changes its workload; a follow-up may
use a newly justified supported Luna or Astra route, but an already running turn
cannot be changed in place.

### 5. Launch the verification gate

Create `Dale Index · Evidence Verifier` only after the three discovery reports
are available. Select its route from the actual contradictions, missing
coverage, repository complexity, and consequence of a false pass—not from the
coordinator's settings or the generic fact that it is a verifier. Give it:

- the raw reports, not the coordinator's preferred synthesis;
- the repository root and applicable instructions;
- the primitive file contract;
- the adversarial verifier prompt from `references/index-roles.md`.

Require it to reject unsupported claims, detect contradictions and stale
documentation, identify missing coverage, and return a pass/fail list. Do not
ask it to make repository edits.

Create the verifier in the same selected project's `local` environment and add
its exact identifiers, status, and cursor to the authoritative manifest.

### 6. Integrate only survivors

Let the calling task write or update all six primitives. Preserve existing
hand-written content unless current evidence disproves it. Mark unresolved
conflicts and unknowns explicitly; never fill a gap with a plausible guess.

For each material claim, include a compact source pointer such as a file path,
symbol, test name, or exact non-secret command. Prefer useful summaries over
inventories of every file.

### 7. Validate the result

Check at minimum:

1. All six primitives exist together.
2. No template markers, TODO placeholders, fake dates, or invented owners remain.
3. Commands and paths referenced by the primitives exist or are labeled
   unverified.
4. `REQ.md` acceptance criteria map to `TDD.md` checks where evidence exists.
5. `CONTEXT.md` and `DECISIONS.md` do not disagree.
6. `STATE.md` describes current state, not a speculative roadmap.
7. `DESIGN.md` reflects the product's actual UI surface or explicitly records
   non-applicability.
8. A final diff or file inventory contains only Dale's intended documentation
   changes.

Do not run broad builds or test suites solely for documentation indexing.
Run a narrow non-mutating command only when it is the cheapest way to verify a
material claim.

## Report completion

State which primitives were created or refreshed, which claims remain unknown,
what the verifier rejected, and the exact checks run. Include one
`::created-thread` directive per created task with a real returned `threadId`.
Do not emit a directive for a `clientThreadId`-only or uncertain result.
