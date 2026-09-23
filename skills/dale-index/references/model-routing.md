# Luna and Sol routing for Dale Index

Dale Index chooses a route per visible task. Its automatic catalog contains
only Luna and Sol.

## Runtime catalog

Immediately before creating tasks, inspect the live `create_thread` schema. It
is authoritative for available models and supported reasoning efforts on the
calling host; the destination host validates the final combination.

| Route | Model ID | Indexing use | Normal thinking range |
|---|---|---|---|
| Luna standard | `gpt-6-luna` | Bounded repository discovery, inventory, extraction, and narrow evidence checks | `high` |
| Luna deep | `gpt-6-luna` | Broad or coupled repository analysis, contradiction resolution, and ordinary adversarial verification | `high` or `max` |
| Sol | `gpt-6-sol` | Genuinely difficult cross-layer synthesis or high-stakes verification where stronger reasoning can change the result | `high` |

Dale routes Luna only at `high` or `max`. Validate the chosen effort against
the live schema. Sol efforts are also validated against the live schema. If no allowed
Luna or Sol route is available, omit automatic overrides and report the
fallback instead of inventing a model ID. If the user required a specific route,
report the conflict and do not substitute it silently.

## Route each role from evidence

For every task:

1. Apply explicit supported Luna or Sol constraints from the current request.
2. Estimate scope size, coupling, ambiguity, evidence quality, and cost of a
   false conclusion.
3. Start discovery on Luna at `high`. Raise an individual discovery
   role to `max` only when its actual scope requires substantial
   cross-file or cross-layer reasoning.
4. Start the verifier on Luna with `high` or `max` for broad or coupled
   analysis. Use Luna at `high` for a small, mechanically checkable
   repository; promote to Sol only when the collected reports expose a
   consequential ambiguity or difficult cross-layer conflict.
5. Pick effort separately: `high` for mechanical inventory and normal
   repository reasoning, and `max` for complex analysis or proof.
6. Show and record one concrete routing reason per task.

Discovery tasks must not default to Sol. A verifier does not automatically need
Sol. Never assign `xhigh`, `max`, or `ultra` merely because the coordinator uses
that setting. Use Sol `xhigh` only for an unusually difficult, consequential
reasoning bottleneck that can be named; use Luna `max` when the owned analysis
or proof obligation benefits from it, without requiring a prior failure. Route
`ultra` only to Sol and only when the live schema confirms that combination.

Model choice may change for an undispatched verifier or a later follow-up when
new evidence materially changes the workload. Record the new reason. Do not
claim to alter an already running turn.
