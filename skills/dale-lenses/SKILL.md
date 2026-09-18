---
name: dale-lenses
description: Reframe a bounded hard question with one falsifiable reasoning lens. Use when a different representation could materially change the answer.
---

# Dale Lenses

Choose one method whose required artifact can change the next decision. Add one
opposing lens only when it makes a different prediction or could reverse the
conclusion.

## Bound the method

- Preserve the requested phase: analysis does not authorize planning or
  execution. Stay in the current task unless evidence gathering is separately
  authorized.
- Require a bounded question, system boundary, time horizon, consequence of
  error, and available evidence. Use ordinary brainstorming while the goal or
  options are still forming.
- Separate observations, inferences, assumptions, preferences, and unknowns.
  Never invent telemetry, probabilities, causality, constraints, or boundaries.
- Honor a user-named lens when it fits. Explain a mismatch and offer the nearest
  useful method instead of silently substituting one.
- Skip the skill when ordinary reasoning is sufficient.

## Select the lens

| Lens | Use when uncertainty concerns | Artifact |
|---|---|---|
| causal | competing explanations or interventions | rival models and a discriminating observation |
| systems | feedback, delays, recurrence | bounded feedback model and predictions |
| temporal | ordering, retries, concurrency, TTL | state transitions and an interleaving trace |
| conservation | missing or duplicated units or authority | balance equation and residual |
| invert | consequential failure paths | failure recipe, controls, and tripwires |
| constraint | throughput resists local optimization | system constraint and migration signal |
| contradiction | necessary requirements appear coupled | exact coupling and separation test |
| decide | costly uncertainty or irreversibility | options, regret, reversibility, value of information |

After selecting, read only that lens's section in
[lens contracts](references/lens-contracts.md). Do not load the other lens
contracts unless a genuinely opposing lens is also selected.

## Apply and stop

State the chosen method, artifact, and terminal condition before interpreting
new evidence. Construct the artifact, preserve rival models and negative
evidence, and fix the falsifier or update threshold before drawing a conclusion.
Prefer the cheapest safe observation that distinguishes live models.

Return the bounded question, evidence boundary, selected lens, model, decision
cruxes, terminal test, decision impact, and material unknowns. Use
`unidentified` when accessible evidence cannot distinguish the models and `not
applicable` when the lens's units, boundary, or falsifier cannot be defined.

Stop when the lens no longer changes the model, evidence need, or decision. Do
not append a plan or begin execution unless the user asks for that next action.
