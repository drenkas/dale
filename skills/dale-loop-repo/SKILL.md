---
name: dale-loop-repo
description: Maintain a repository on a recurring heartbeat by inspecting live priorities and dispatching bounded verified work. Use for recurring engineering maintenance.
---

# Dale Loop Repo

Run a conservative maintainer heartbeat. Find useful machine-checkable work, delegate it, and leave the repository cleaner than the previous wakeup.

## Workflow

1. Read repository instructions and identify its active branch, local notes, roadmap, open issues and PRs, recent CI, and existing maintainer state.
2. Define the cadence, priority sources, verification gates, concurrency limit, merge authority, and terminal condition. Default to one active implementation lane and one review lane.
3. On each wakeup, refresh live state before choosing work. Prefer already-started or blocking work over generating new work.
4. Select only bounded, in-scope tasks with objective verification. Avoid vague product work, architecture rewrites, auth, payments, deployments, and destructive changes unless explicitly authorized.
5. Create fresh user-visible Codex tasks in the exact saved project's `local`
   environment; do not create worktrees. Serialize all mutating tasks and
   subagents per repository, including disjoint paths. Read-only review may run
   concurrently; keep implementation and independent review separate.
6. Track task ids, branches, PRs, attempts, last observation, and next action. Do not dispatch duplicate work for unchanged state.
7. Carry owned PRs through checks and review, using `$dale-loop-pr` behavior and
   only the Git or external actions the user authorized. Observe-only
   maintenance stays read-only.
8. Stop or pause the heartbeat through the available automation tool when the
   terminal condition holds. Escalate repeated blockers with evidence.

Return a short report after meaningful action and stay quiet on unchanged
wakes. Do not archive the calling task unless the user asks.

## Task registry

Persist an authoritative registry in the coordinator conversation and every
continuation summary: exact returned `threadId`, `hostId`, `projectId`, title,
task status, and latest wait cursor. Use these IDs directly for read, send, and
wait operations. `list_threads` is optional discovery only and cannot gate a
known ID or prove absence. Validate a user-supplied actual ID with `read_thread`
before adoption. Never use a `clientThreadId` as a real ID; a client-only or
ambiguous creation is uncertain and must never be recreated.
