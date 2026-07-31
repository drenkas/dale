---
name: dale-loop-goal
description: Keep one agent task progressing linearly until an objective completion condition passes, with fresh state checks and hard resource limits. Use when the user has one long-running goal that does not need parallel PR orchestration or a changing workflow graph.
---

# Dale Loop Goal

Keep one thread moving toward one outcome. Do not turn it into a swarm unless the work genuinely branches.

## Workflow

1. State the outcome as a verifiable predicate and record the current evidence.
2. Set iteration, elapsed-time, and repeated-failure limits plus any required human gates.
3. Work on the smallest next action that can change the predicate.
4. At the end of every turn, use fresh evidence to evaluate completion. Do not accept self-reported completion without a check.
5. If incomplete and progress was made, persist the new state and continue through the product's goal or wakeup mechanism.
6. If incomplete and the same failure repeats without new evidence, change approach once, then escalate at the configured limit.
7. Stop immediately for permissions, destructive requirements, ambiguous intent, or a human approval gate.
8. Finish with the predicate, evidence, iterations, elapsed time, work completed, and remaining blocker.

Prefer this shape for one deep task. Use `$dale-loop-project` when the goal becomes several independently reviewable work streams.
