# GPT-5.6 routing for Dale Index

Dale Index chooses a route per visible task. Its automatic catalog contains
only the GPT-5.6 generation.

## Runtime catalog

Immediately before creating tasks, inspect the live `create_thread` schema. It
is authoritative for available models and supported reasoning efforts on the
calling host; the destination host validates the final combination.

| Route | Model ID | Indexing use | Normal thinking range |
|---|---|---|---|
| Luna standard | `gpt-5.6-luna` | Bounded repository discovery, inventory, extraction, and narrow evidence checks | `low` to `medium` |
| Luna deep | `gpt-5.6-luna` | Broad or coupled repository analysis, contradiction resolution, and ordinary adversarial verification | `xhigh` to `max` |
| Sol | `gpt-5.6-sol` | Genuinely difficult cross-layer synthesis or high-stakes verification where stronger reasoning can change the result | `high` |

At authoring time, Luna accepts `low`, `medium`, `high`, `xhigh`, and `max`;
Sol also accepts `ultra`. The live schema outranks this snapshot. If no allowed
GPT-5.6 route is available, omit overrides and report the fallback instead of
inventing a model id.

## Route each role from evidence

For every task:

1. Apply explicit supported GPT-5.6 constraints from the current request.
2. Estimate scope size, coupling, ambiguity, evidence quality, and cost of a
   false conclusion.
3. Start discovery on Luna at `low` or `medium`. Raise an individual discovery
   role to `xhigh` or `max` only when its actual scope requires substantial
   cross-file or cross-layer reasoning.
4. Start the verifier on Luna with `xhigh` or `max` for broad or coupled
   analysis. Use Luna at lower effort for a small, mechanically checkable
   repository; promote to Sol only when the collected reports expose a
   consequential ambiguity or difficult cross-layer conflict.
5. Pick effort separately: `low` for mechanical inventory, `medium` for normal
   repository reasoning, and `xhigh` or `max` for complex analysis or proof.
6. Show and record one concrete routing reason per task.

Discovery tasks must not default to Sol. A verifier does not automatically need
Sol. Never assign `xhigh`, `max`, or `ultra` merely because the coordinator uses
that setting. Use `xhigh` only for an unusually difficult, consequential
reasoning bottleneck that can be named; use `max` only after a cheaper route
failed or a uniquely high-stakes proof obligation clearly justifies it. Route
`ultra` only to Sol and only when the live schema confirms that combination.

Model choice may change for an undispatched verifier or a later follow-up when
new evidence materially changes the workload. Record the new reason. Do not
claim to alter an already running turn.
