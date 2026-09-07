---
name: dale-loop-project
description: Orchestrate a multi-stage engineering project through dynamically grouped or stacked Codex tasks, branches, pull requests, independent reviews, remediation cycles, and merge. Use when one goal needs multiple PRs or when the user wants a project carried autonomously from plan to merged changes.
---

# Dale Loop Project

Turn one engineering goal into a dependency-aware PR graph and carry it to merge.

## Workflow

1. Read repository instructions, confirm GitHub access, resolve the default branch, preserve unrelated changes, and discover canonical checks.
2. Split work by independently reviewable outcome. Group coupled small changes; separate risky or independently shippable work. Mark parallel, sequential, and stacked dependencies.
3. Show a ledger with scope, dependency, branch, verifier, state, and attempt count.
4. Create each ready task in the exact saved project's `local` environment; do
   not create worktrees. Because tasks share the branch, index, and working
   tree, run at most one mutating task or subagent per repository at a time,
   including for disjoint paths. Require a `codex/` branch, behavioral
   verification where possible, repository checks, commit, push, and ready PR.
   Before each writer starts, verify the checkout is clean enough to transition
   without losing user work and is on the intended starting state: the default
   base for a new independent unit, its predecessor branch for a new stacked
   unit, or the existing verified task/PR branch for remediation or resumed work. Never
   reset, stash, or switch away dirty user changes. If the required transition
   is not safe and authorized, block that unit.
5. Reconcile every report with live GitHub state. When a PR head first appears or changes, create a fresh independent review task with the acceptance criteria, diff, and verification evidence.
6. Create a fresh remediation task for active valid findings. Re-review the new head. Stop after three unsuccessful cycles or an earlier ambiguous/destructive blocker.
7. Treat a PR as clean only when required checks pass, independent review has no actionable findings, GitHub has no unresolved thread or active change request, the diff is in scope, and the PR is mergeable.
8. Squash-merge clean PRs in dependency order and delete remote branches. Update or retarget dependent work against the merged base, then rerun checks and review.
9. Finish with task ids, branches, PRs, checks, review cycles, merge commits, and blockers.

Never let an implementation task approve its own work. Never merge silence as approval.

## Task registry

Persist an authoritative registry in the coordinator conversation and every
continuation summary: exact returned `threadId`, `hostId`, `projectId`, title,
unit status, and latest wait cursor. Use these IDs directly for read, send, and
wait operations. `list_threads` is optional discovery only and cannot gate a
known ID or prove absence. Validate a user-supplied actual ID with `read_thread`
before adoption. Never use a `clientThreadId` as a real ID; a client-only or
ambiguous creation is uncertain and must never be recreated.
