---
name: dale-lenses
description: Reframe an already bounded, difficult engineering, product, operational, or strategic question through one rigorous reasoning lens and at most one genuinely opposing lens, producing a falsifiable model, decision cruxes, and evidence needs instead of a tour of frameworks. Use when the user explicitly invokes $dale-lenses, selects Dale Lenses through /skills, or directly asks to apply causal, systems, temporal, conservation, inversion, constraint, contradiction, or decision-theoretic thinking to a defined question. Do not use for routine implementation, open-ended brainstorming that is still forming the question or options, or a straightforward problem whose answer does not depend on reframing.
---

# Dale Lenses

Change the representation of a hard problem before changing the answer. Apply a
specific epistemic method with an observable artifact and a stop rule; never
simulate rigor by naming several frameworks and repeating the same opinion.

## Keep the method bounded

- Preserve the user's current phase. Analyze, decide, plan, or implement only to
  the extent requested; invoking a lens does not authorize execution.
- Require a bounded question. When the user is still discovering the goal,
  options, taste, or priorities, keep or move the conversation to ordinary
  brainstorming instead of imposing a lens prematurely.
- Stay in the current task. Do not create Codex tasks or subagents merely to
  manufacture perspectives. Use tools only when actual evidence is needed and
  already authorized.
- Separate observations, inferences, assumptions, preferences, and unknowns.
- Redact secrets, credentials, raw environment values, personal data, private
  transaction identifiers, and other sensitive operational evidence before it
  enters a tool call, artifact, or response.
- Select one primary lens. Add one challenging lens only when it makes a
  materially different prediction, exposes a different failure mode, or could
  reverse the decision.
- Honor a lens named by the user. If it does not fit the problem, explain the
  mismatch briefly and offer the closest fitting lens instead of silently
  substituting it.
- Decline the method when ordinary reasoning is sufficient. “No useful
  reframe” is better than framework theatre.

## Select by problem signature

Read `references/lens-contracts.md` completely before applying a lens.

| Lens | Use when the core uncertainty concerns | Required artifact |
|---|---|---|
| `causal` | competing explanations or interventions | rival causal models and a discriminating observation |
| `systems` | feedback, delays, recurrence, or second-order effects | bounded feedback model with testable predictions |
| `temporal` | retries, ordering, concurrency, TTL, or version skew | state transitions and a concrete interleaving trace |
| `conservation` | missing or duplicated money, events, jobs, inventory, or authority | scoped balance equation and residual |
| `invert` | consequential failure paths or ineffective safeguards | minimal failure recipe, controls, and tripwires |
| `constraint` | throughput stays flat despite local optimization | evidenced system constraint and migration signal |
| `contradiction` | two necessary requirements appear mutually exclusive | exact coupling and a separation that tests both sides |
| `decide` | costly choice under uncertainty or irreversibility | options, states, regret, reversibility, and value of information |

Do not select by novelty. Select the lens whose required artifact could most
change the next decision.

## Run the lens

### 1. Bound the question

State the exact question or decision, system boundary, time horizon,
consequence of error, and evidence currently available. Resolve an ambiguous
scope from local context when safe; ask at most one question only when different
answers would materially change the lens or decision.

### 2. Precommit the method

Name the primary lens and its concrete reason in one line. If using a challenging
lens, state what different prediction or failure mode justifies it. Define the
artifact and method-specific terminal condition before drawing a conclusion.

### 3. Construct, then attack

Build the artifact required by the selected lens. Preserve rival models and
negative evidence. Record the strongest counterargument to the leading model
before supporting it. Fix the falsifier, invalidation threshold, update trigger,
or boundary of identification before interpreting any new result; do not move
the gate after seeing evidence.

When evidence can be gathered safely, prefer the cheapest observation that
distinguishes live models. Do not ask for measurements that cannot affect the
decision. Never invent telemetry, probabilities, causality, constraints, or
system boundaries.

### 4. Expose the crux

Compress the result into a decision-relevant crux map:

```text
question: <bounded question>
evidence boundary: <observed sources and unavailable surfaces>
primary lens: <lens and why>
challenging lens: <lens and distinct contribution, or none>
model: <required lens artifact>
cruxes: <assumptions or observations that can change the conclusion>
terminal test: <prediction, threshold, counterexample, or update/kill trigger>
status: <method-specific terminal state justified by evidence and preferences>
decision impact: <what changes now, if anything>
unknowns: <material unresolved items>
```

Adapt the presentation to the problem rather than forcing a table when prose or
a small diagram is clearer. Keep the fields and epistemic distinctions even
when the rendering changes.

### 5. Stop honestly

Stop when the chosen lens no longer changes the model, evidence need, or
decision. Return `unidentified` when accessible evidence cannot distinguish the
live models. Return `not applicable` when the lens's required units, boundary,
or falsifier cannot be defined. Do not synthesize a compromise merely because
two lenses disagree; expose the disagreement and the evidence that would settle
it.

## Hand off without expanding scope

Answer the user's requested decision or analysis at the end, preserving the
strongest counterargument and remaining uncertainty. Do not append an
implementation plan, create an artifact file, invoke another Dale skill, or
start execution unless the user explicitly asks for that next action.
