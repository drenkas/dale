# Luna and Sol routing for Dale Index

Dale Index chooses a route per independent worker. Its default catalog contains
Luna and Sol; an explicit user model or effort supported by the live schema
takes priority.

## Runtime catalog

Immediately before delegation, inspect the live `spawn_agent` schema. Use
`model` and `reasoning_effort` for an explicit override, with `fork_turns: "none"`
and a self-contained role prompt. Do not inherit the coordinator's route by
accident. For separate chats explicitly requested by the user, inspect
`create_thread` independently and use its `model` and `thinking` fields; the
destination host validates the final combination.

| Route | Model ID | Indexing use | Normal thinking range |
|---|---|---|---|
| Luna standard | `gpt-6-luna` | Bounded repository discovery, inventory, extraction, and narrow evidence checks | `high` |
| Luna deep | `gpt-6-luna` | Broad or coupled repository analysis, contradiction resolution, and ordinary adversarial verification | `high` or `max` |
| Sol | `gpt-6.1-sol` | Genuinely difficult cross-layer synthesis or high-stakes verification where stronger reasoning can change the result | `high` |

Dale's default Luna routes use only `high` or `max`. Validate every model/effort
combination against the live schema. If a requested route is unavailable, report
the conflict without silently substituting. If only automatic delegation is
unavailable, inspect directly and disclose the actual route and evidence limits;
do not invent a model ID or pretend direct work used the intended worker route.

## Route each role from evidence

For every worker:

1. Apply explicit supported model or effort choices from the current request.
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
6. Record one concrete routing reason per worker.

Discovery tasks must not default to Sol. A verifier does not automatically need
Sol. Never assign `xhigh`, `max`, or `ultra` merely because the coordinator uses
that setting. Use Sol `xhigh` only for an unusually difficult, consequential
reasoning bottleneck that can be named; use Luna `max` when the owned analysis
or proof obligation benefits from it, without requiring a prior failure. In
automatic routing, use `ultra` only for Sol when the live schema confirms that
combination. Repair an unclear work contract before spending more reasoning.

Model choice may change for an undispatched verifier or a later follow-up when
new evidence materially changes the workload. Record the new reason. Do not
claim to alter an already running turn.
