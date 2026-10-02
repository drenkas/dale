# Luna and Sol routing for brainstorm evidence

Dale Brainstorm stays one-on-one. This router applies only to a substantial,
separable factual investigation. Its default catalog contains Luna and Sol;
an explicit user model or effort supported by the live schema takes priority.

Inspect the live `spawn_agent` schema before delegation. Use `model` and
`reasoning_effort` for an explicit override, with `fork_turns: "none"` and a
self-contained evidence prompt. Do not inherit the coordinator's route by
accident. Use `create_thread` with `model` and `thinking` only when the user
explicitly requests a separate chat; inspect that tool's schema independently.

| Route | Model ID | Evidence-task use | Normal thinking range |
|---|---|---|---|
| Luna standard | `gpt-6-luna` | Bounded lookup, repository sweep, extraction, or narrow measurement | `high` |
| Luna deep | `gpt-6-luna` | Coupled investigation, technical comparison, or evidence that needs substantial interpretation | `high` or `max` |
| Sol | `gpt-6.1-sol` | Rare, consequential ambiguity where deep cross-domain reasoning can materially change the evidence conclusion | `high` |

Dale's default Luna routes use only `high` or `max`. Validate every model/effort
combination against the live schema. If a requested route is unavailable, report
the conflict instead of silently substituting. If only automatic routing is
unavailable, inspect directly and disclose the actual route and evidence limits;
do not invent a model ID or pretend direct work used the intended worker route.

Choose the cheapest sufficient route. Start with Luna `high`, raise it to `max`
for real coupling or interpretation, and use Sol for a concrete high-consequence
reason. Record the model, effort, and workload reason for a delegated check.

Use Sol `xhigh` only for a named, unusually difficult reasoning bottleneck.
Use Luna `max` when the evidence analysis genuinely benefits from deeper
reasoning; no failed cheaper attempt is required. In automatic routing, use
`ultra` only for Sol and only when the live schema confirms that combination.
Repair an unclear evidence contract before spending more reasoning.
