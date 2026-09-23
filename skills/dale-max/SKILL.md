---
name: dale-max
description: Run an explicit-only resident worker and review loop for consequential implementation, debugging, migration, audit, or delivery work.
---

# Dale Max

Own one consequential outcome through implementation, independent challenge,
revision, and direct proof. Keep persistent resident threads for continuity;
add ephemeral reviewers only when a distinct risk or evidence question warrants
one.

## Authority

- Explicit invocation authorizes the visible Codex tasks required by this loop
  and task-matched Luna/Sol routing. It does not authorize broader product
  scope, external writes, Git history changes, deployment, or deletion.
- Keep created tasks visible and user-owned. Do not archive or hand them off
  unless asked. Created tasks must not create other visible Codex tasks.
- Use one mutating writer per shared checkout. Read-only reviewers may run in
  parallel; route requested edits back to the resident Worker.
- Prefer a saved project's local checkout. Use projectless tasks only when no
  repository is involved.
- If task creation is unavailable, report the blocker rather than silently
  substituting another orchestration mode.
- Explicit invocation grants intent, not tool capability; obey live task and
  model schemas when dispatching.

Read [node contracts](references/contracts.md) before dispatch. Read
[model routing](references/model-routing.md) when choosing or changing a route.

## Shape the run from the proof obligation

Define the requested outcome, authority, acceptance criteria, forbidden
outcomes, and primary observable signal. Inspect repository instructions and
live state before writing a plan. Use the smallest graph that can produce and
challenge the result:

- a persistent Resident Worker owns implementation and integration;
- a persistent Lead Reviewer owns independent review and synthesis;
- an ephemeral Plan Reviewer is useful only when a mistaken plan would be
  costly;
- additional ephemeral reviewers each own one concrete risk, boundary, or
  falsification question.

Do not invent a reviewer count, cycle count, token spend, or model tier. Honor
budgets and limits supplied by the user. Add a node only when its answer can
change the work or verdict. Redact credentials, tokens, raw environment values,
private keys, and unnecessary personal data from ledgers and task prompts.

## Create and track resident tasks

Resolve the exact saved project with `list_projects`. Create the Worker and Lead
Reviewer with complete initial prompts, explicit routes, titles, ownership,
evidence requirements, and stop conditions. Fork persistent successors only
when continuity is needed and the source task has completed history.

Maintain the run ledger defined in `references/contracts.md`. Persist every
returned `threadId`, `hostId`, route, status, and wait cursor immediately. A
known real ID is authoritative; listings are discovery aids, not proof of
absence. Never pass a `clientThreadId` where a `threadId` is required. Ambiguous
creation blocks recreation unless the service proves that no task was created.

## Run the control loop

1. Let the Worker inspect, plan, and identify the primary signal. If plan risk
   warrants it, ask one Plan Reviewer to challenge concrete assumptions before
   implementation.
2. Let the Worker implement the coherent task and run proportionate checks.
   Keep unrelated user changes intact and keep all repository writes with this
   one owner. Its output already exists in the shared checkout: inspect and
   validate that actual state, never replay or reapply a reported patch.
3. Give reviewers the exact revision, diff, requirements, and evidence boundary.
   Keep their prompts independent: do not pre-state the desired verdict or copy
   another reviewer's conclusion.
4. The Lead Reviewer synthesizes raw reviewer evidence. The coordinator then
   applies the final pass gate to the actual checkout and direct evidence.
   Conflicts remain explicit; majority agreement is not proof.
5. For a failed gate, send the Worker only actionable findings and the failed
   primary signal. After revision, review the complete new result again. If two
   revisions do not move the same signal, reframe the cause or evidence need.

Use `wait_threads` with retained cursors and bounded waits. A timeout with live
progress is not failure. Use `read_thread` only when the wait result omits
material evidence. A route may change on a later follow-up when the workload
changes; never claim to reroute a turn already running.

## Completion

Finish only when the coordinator gate returns `PASS`, every acceptance
criterion has direct evidence at the claimed boundary, and every created task
has a real ID and terminal state. Report code/static, test, browser, and
deployed/runtime evidence separately so one level does not imply another.

If a task is blocked or creation remains uncertain, report the exact ID or
diagnostic identifier and reason without claiming completion. Return one
`::created-thread` directive for each successfully created task with a real
`threadId`; never emit one for a client-only or ambiguous result.
