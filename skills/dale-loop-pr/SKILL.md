---
name: dale-loop-pr
description: Carry one existing pull request through review, authorized repair, and fresh-head verification until merge-ready or blocked.
---

# Dale Loop PR

Keep one PR moving without making the user relay comments between GitHub and Codex.

## Workflow

1. Resolve the PR and read repository instructions, scope, checks, active unresolved threads, latest review state, mergeability, and head SHA.
2. Record the head SHA, active findings, check state, cycle count, and last meaningful activity.
3. When the head is new, create a fresh independent review task. Give it the original intent, acceptance criteria, diff, and verification evidence, not the maker's reasoning.
4. When valid findings or PR-caused CI failures exist and repair is authorized,
   create a fresh remediation task on the PR branch. Require scoped fixes and
   behavioral and repository checks; request commit and push only when those
   actions are also authorized.
   Create saved-project tasks in its `local` environment without worktrees.
   Serialize every mutating task or subagent in that repository, even for
   disjoint paths; read-only review may run concurrently. Persist exact returned
   task IDs and wait cursors in the coordinator state and use known IDs directly.
   Never treat a `clientThreadId` as a real ID or recreate uncertain work.
5. Re-observe GitHub after every task. A new head triggers a new review. Do not reuse the maker or previous checker.
6. Declare merge-ready only when required checks pass, no actionable independent finding remains, no unresolved GitHub thread or active change request remains, scope is valid, and the PR is mergeable.
7. Stop after three failed remediation cycles, repeated unchanged failures, conflicting intent, unrelated required work, missing permission, or destructive action.
8. Merge only when the user requested it. Otherwise return a merge-ready receipt.

Treat monitoring, repair, commit, push, review requests, and merge as separate
authorities. An observe-only request stays read-only. Require commit or push
only when the user authorized it; otherwise prepare and verify the local result.

Poll through task wakeups or automations, not shell sleeps. Silence is not approval.
