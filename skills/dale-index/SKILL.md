---
name: dale-index
description: Build or refresh six evidence-backed project primitives. Use for repository mapping, project-context recovery, or a Dale Index request.
---

# Dale Index

Build a compact source of truth from the repository as it exists now. Work
directly for a small repository or narrow refresh; use read-only subagents for
independent evidence boundaries when that saves time or improves coverage.
The calling task is the sole integrator and document writer. No saved Codex
project or separate chat is needed for direct work or subagents.

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

Use the established primitive directory even when its set is incomplete. Add
missing documents there and preserve relative links. Use the repository root
only when no established location exists; resolve competing locations from
repository conventions before writing. Never create a second competing set.

Use the schemas under `assets/primitives/` when creating files. Treat them as
structure, not prose to copy blindly. Remove every placeholder from files
written to the target repository.

## Bound the refresh

1. Resolve the repository root.
2. Read every applicable `AGENTS.md`, including deeper files for scoped paths.
3. Inspect `git status`, tracked files, manifests, lockfiles, runtime entrypoints,
   tests, schemas, routes, configuration examples, and existing documentation.
4. Record the current branch or revision when Git exists. Never mutate Git state.
5. Label repository evidence as `verified`, `inferred`, `unknown`, or `stale`.
6. Treat code, tests, schemas, and runtime output as stronger evidence than docs.

Do not print secrets or raw environment values. Use names of required variables,
not their values.

For an initial index, cover material repository boundaries. For a refresh, use
the last documented revision and actual changes to inspect affected claims and
their consumers, including uncommitted changes. If the baseline is missing or
unreliable, broaden inspection and disclose the coverage limit.

## Discover and verify

Read [index roles](references/index-roles.md) when splitting discovery; combine
or omit their candidate contracts to match real evidence boundaries. Read
[model routing](references/model-routing.md) before dispatch. Give each worker
minimum repository context, read-only scope, one structured evidence report,
and no further delegation. Continue a distinct inspection or integration
preparation while workers run. Workers never edit the shared checkout.

Distinguish implemented behavior from exercised behavior. Keep a compact source
pointer for every material claim: path, symbol, test, or non-secret command.

For consequential contradictions or a broad initial synthesis, use a fresh
read-only verifier. Select its route from evidence risk, not the generic role
name. Give it:

- the raw reports, not the coordinator's preferred synthesis;
- the repository root and applicable instructions;
- the primitive file contract;
- the adversarial verifier prompt from [index roles](references/index-roles.md).

Require it to reject unsupported claims, detect contradictions and stale
documentation, identify missing coverage, and return a pass/fail list. Do not
ask it to make repository edits.

For a small refresh, check affected claims directly. If delegation or independent
verification is unavailable, finish supported documentation and disclose the
missing independence; do not invent a verifier or block accessible indexing
merely because task tools are absent.

## Separate chats, only when requested

Create discovery chats only when the user explicitly requests separate chats.
Call `list_projects`, resolve the exact saved project, and use its `local`
environment unless the user explicitly requests a worktree; resolve an ambiguous
project before creation. Keep every
chat read-only, user-owned, and forbidden from further delegation. Do not archive
it unless asked.

Before dispatch, record title, exact `projectId`, host, role, route, reason, and
`dispatching` status. Persist real returned `threadId`, `hostId`, status, and wait
cursor in the manifest and continuation summaries. Use known IDs directly for
read/send/wait; `list_threads` cannot gate a known ID or prove absence. Validate
user-supplied IDs with `read_thread`. Never use a `clientThreadId` as a real task
ID or recreate an ambiguously created chat.

Collect explicitly requested chats with one bounded `wait_threads` call, reusing
`afterCursor`. A progress timeout is normal. Read a chat only when compact
output lacks material evidence; send a precise contract correction when needed.
Keep route changes for subsequent turns; never claim to alter a running turn.

## Integrate only survivors

Let the calling task write or update all six primitives. Preserve existing
hand-written content unless current evidence disproves it. Mark unresolved
conflicts and unknowns explicitly; never fill a gap with a plausible guess.

For each material claim, include a compact source pointer such as a file path,
symbol, test name, or exact non-secret command. Prefer useful summaries over
inventories of every file.

## Validate and report

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

State which primitives were created or refreshed, which claims remain unknown,
what the verifier rejected, and the exact checks run. Record revision and
coverage so the next refresh can be scoped. Follow the live tool's
`::created-thread` directive with `threadId` for ready creation or `clientThreadId`
only for explicitly reported queued setup. Never label a queued ID as real or
emit a directive for an ambiguous result.

## Cancellation and changed scope

On cancellation or revoked scope, interrupt affected subagents immediately,
invalidate pending outputs, and preserve existing changes. For a correction,
stop conflicting work first, send the revised contract, and accept results only
against the current request and artifact state.
