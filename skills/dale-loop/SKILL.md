---
name: dale-loop
description: Design and run a loop around any repeatable agent workflow. Use when the user invokes `$dale-loop`, asks Codex to keep watching or working, wants agents to prompt other agents, or needs a dynamic workflow whose shape should be inferred from the goal rather than selected up front.
---

# Dale Loop

Turn a goal into a closed loop that can keep moving without the user carrying context between steps.

Use this universal shape:

```text
observe -> decide -> dispatch -> verify -> persist -> wait or stop
```

Do not force every problem into a PR pipeline. Design the smallest loop that matches the work.

## 1. Define the contract

Extract or determine:

- **Outcome:** the state that must become true
- **Observation:** the live sources that reveal current state
- **Trigger:** an event, cadence, completed task, changed commit, or failed check
- **Actions:** what Codex may do next
- **Verifier:** objective evidence that accepts or rejects progress
- **State:** what must survive between wakeups
- **Limits:** time, iterations, child tasks, cost signals, and repeated failures
- **Human gates:** actions or judgment calls that still require the user

Ask only when a missing answer materially changes authority or the outcome. Never use “the agent says it is done” as the only verifier.

## 2. Choose the loop shape

Route to a focused skill when one shape clearly fits:

- `$dale-loop-project`: split a project across dependent or parallel PRs and carry them through merge
- `$dale-loop-pr`: watch and repair one existing PR
- `$dale-loop-repo`: wake on a cadence, inspect a repository, and direct useful work to fresh tasks
- `$dale-loop-watch`: monitor a source or condition and notify or act only on meaningful change
- `$dale-loop-goal`: keep one task moving linearly until an objective check passes

For mixed or novel work, generate a bespoke graph. Create only the workers, reviewers, waits, and nested loops the current state requires. Do not predefine agent personas.

Show the chosen shape in a compact ledger before launching it. Continue without waiting unless the proposed shape introduces material ambiguity or new authority.

## 3. Run the loop

1. Read applicable instructions and inspect live state.
2. Create user-visible Codex tasks for independent work. Use isolated worktrees for concurrent repository changes.
3. Give every task the goal, current state, scope, verifier, limits, and required return fields.
4. Reconcile task reports with live systems. Live GitHub, CI, deployments, trackers, and monitored sources beat stale reports.
5. Persist the minimum state needed to resume: work units, task ids, dependencies, attempts, evidence, last observation, next wake, and blockers.
6. On each wakeup, re-observe before acting. Do not replay stale actions.
7. Create a fresh checker when independent judgment matters. Do not let a maker approve its own output.
8. Stop or escalate when the verifier passes, a human gate is reached, limits are exhausted, or the same blocker repeats without new evidence.

Use the product's task wakeup or automation mechanism for cadence-based loops. Do not hold a blocking shell sleep. Archive or end scheduled work when the terminal condition is reached.

## 4. Return a receipt

Report the loop shape, tasks and nested loops created, state transitions, verification evidence, resource/attempt counts, completed actions, preserved blockers, and next human decision. Never claim completion from child-task prose alone.
