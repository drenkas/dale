---
name: dale-loop-goal
description: Keep one long-running task moving until an objective completion check passes or a configured limit blocks progress. Use when the goal is linear and does not need parallel PR or graph orchestration.
---

# Dale Loop Goal

Keep one thread moving toward one outcome. Do not turn it into a swarm unless the work genuinely branches.

## Workflow

1. State the outcome as a verifiable predicate and record the current evidence.
2. Honor user or tool limits; otherwise set proportionate iteration, elapsed-time, and repeated-failure limits plus any required human gates.
3. Work on the smallest next action that can change the predicate.
4. At the end of every turn, use fresh evidence to evaluate completion. Do not accept self-reported completion without a check.
5. If incomplete and progress was made, persist the new state and continue through the product's goal or wakeup mechanism.
6. If incomplete and the same failure repeats without new evidence, change approach once, then escalate at the configured limit.
7. Stop dependent work only for genuinely missing authority, a destructive requirement, material ambiguity, or a human gate. Continue safe independent work and honor authority already granted.
8. Finish with the predicate, evidence, iterations, elapsed time, work completed, and remaining blocker.

Prefer this shape for one deep task. Use `$dale-loop-project` when the goal becomes several independently reviewable work streams.
