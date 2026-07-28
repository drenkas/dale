# Dale Lenses contracts

Use this catalog to enforce method-specific artifacts. Apply one primary lens
and at most one challenging lens. Every lens must expose its evidence boundary
and stop when it no longer changes the decision. Empirical lenses must make a
risky prediction or falsifiable claim; the decision lens must precommit update
and kill triggers without pretending preferences are truth claims.

## Causal

Use for competing explanations, suspected causes, and intervention choices.

1. Name the outcome, candidate causes, mediators, confounders, selection effects,
   and relevant controls.
2. Separate temporal precedence and correlation from a causal mechanism.
3. Preserve at least one credible rival model.
4. Write different predictions for observation versus intervention.
5. Select the cheapest safe observation or intervention that distinguishes the
   live models.

Required artifact: a bounded causal graph or explicit edge list, rival models,
and a discriminating test. Use `supported`, `weakened`, `falsified`, or
`unidentified`; never claim identification from chronology alone.

Stop when one model survives a precommitted discriminator, or when accessible
evidence leaves the models observationally equivalent.

## Systems

Use when a problem recurs, local fixes create side effects, or feedback and
delay dominate behavior.

1. Define the system boundary and measurable outcome.
2. Identify stocks, inflows, outflows, reinforcing loops, balancing loops,
   delays, thresholds, and actors with materially different incentives.
3. Explain the observed dynamic through the smallest sufficient loop set.
4. Derive a time-shaped prediction for each live model.
5. Identify a leverage point only when changing it should move the system-level
   signal, not merely a local metric.

Required artifact: a causal-loop or stock-flow model, boundary, delays,
predictions, and leverage hypothesis.

Stop when the smallest model explains the dynamic and a measurement can test
it. Return `not applicable` if the result is a local owning-layer defect with no
material feedback behavior.

## Temporal

Use for retries, queues, caches, TTL, concurrency, scheduling, replication,
migrations, or version skew.

1. Name relevant clocks: event time, processing time, wall time, lease or TTL,
   and deployment/version time.
2. Define states, legal transitions, identities, idempotency boundaries, and
   ordering guarantees.
3. Construct the minimal interleaving that could produce the symptom.
4. Distinguish steady-state correctness from rollout, retry, recovery, and
   partial-failure states.
5. Test a trace against code, logs, or a safe reproducer whenever available.

Required artifact: state transitions, a concrete ordered or partially ordered
trace, violated invariant, and the observation that would rule the trace out.

Stop at a reproducible witness or when every critical transition in the bounded
model has direct evidence. A timeline without identities and transitions is not
a temporal analysis.

## Conservation

Use when countable units appear, disappear, duplicate, or exceed authority:
money, events, jobs, inventory, quotas, references, or permissions.

1. Define exactly one unit, identity key, accounting boundary, and interval.
2. Separate stocks from flows and gross values from net values.
3. Write the balance equation before examining candidate explanations.
4. Compute the signed residual and trace it through source, sink, duplication,
   reconciliation, and boundary-crossing categories.
5. Do not name a culprit until item-level or aggregate evidence localizes the
   unexplained residual.

Required artifact: `closing = opening + inflows - outflows + adjustments`, with
explicit categories, residual, and next drill-down key.

Stop when the residual is fully explained or localized to the smallest unknown
source or sink. Return `not applicable` when unit, identity, or boundary cannot
be made coherent.

## Invert

Use for consequential changes, repeated failed fixes, controls whose value is
unclear, or pre-mortem analysis.

1. State the failure outcome precisely.
2. Construct the minimal reachable recipe that would guarantee or strongly
   induce that failure.
3. Identify necessary preconditions, trust boundaries, controls, detection
   signals, and recovery limits along the path.
4. Attempt only safe, authorized, reversible falsification. Keep destructive or
   privacy-sensitive paths as bounded thought experiments unless the user grants
   exact additional authority.
5. Derive tripwires and a control test from the minimal path.

Required artifact: minimal failure path, prerequisites, blocking controls,
detection tripwires, and a safe test or explicit untested boundary.

Stop when every modeled reachable path inside the declared boundary is cut by
an evidenced control, or when one modeled path remains live. Preserve unmodeled
or external attack surface as `unidentified`; never infer global safety from a
bounded inversion. A generic risk list is not inversion.

## Constraint

Use when demand exceeds throughput, queues persist, or local improvements do not
move the end-to-end outcome.

1. Define the unit of flow, system objective, demand rate, and observation
   interval.
2. Map stage capacity, queue, utilization, variability, rework, blocking, and
   starvation using evidence rather than intuition.
3. Treat the likely constraint as a hypothesis. Name data that would falsify it.
4. Distinguish exploiting the constraint, subordinating other stages, elevating
   capacity, and changing demand.
5. Predict where the constraint will migrate if the intervention succeeds.

Required artifact: Constraint Case with unit, throughput target, evidenced
bottleneck, invalidation signal, experiment, buffer, and migration signal.

Stop when an intervention moves the global signal or evidence places the
constraint elsewhere. Never recommend optimizing a non-constraint solely
because it is expensive or visibly slow.

## Contradiction

Use when two requirements seem mutually exclusive, such as consistency versus
latency, isolation versus sharing, or safety versus delivery speed.

1. State both required properties with measurable thresholds.
2. Locate the exact resource, coupling, layer, or condition that creates the
   tradeoff.
3. Check whether the conflict can be separated by time, space, condition,
   identity, representation, or system layer.
4. Generate only candidates grounded in available resources and hard bounds.
5. Test both thresholds; do not declare success by quietly weakening one side.

Required artifact: contradiction boundary, hard bounds, separation mechanism,
candidate, and paired acceptance test.

Stop when one candidate satisfies both thresholds or evidence shows the
conflict is fundamental within the declared boundary.

## Decide

Use for costly, uncertain, or partly irreversible choices.

1. Define options, relevant states of the world, consequences, time horizon,
   reversibility, and the cost of delay.
2. Separate facts from preferences and asymmetric losses.
3. Use probability ranges or ordinal confidence unless real frequencies justify
   point estimates.
4. Compare regret and robustness across plausible ranges, not only a single
   expected value.
5. Estimate value of information by whether a feasible observation could change
   the choice more than it costs.
6. Precommit update, abort, or kill triggers for the selected option.

Required artifact: Decision Dossier containing options, world states,
consequence ranges, preferences, regret, reversibility, useful information, and
update triggers.

Use `robust` when one option survives every plausible range,
`information-sensitive` when a feasible observation could reverse it,
`preference-bound` when unresolved values determine the choice, and `deferred`
when the current boundary cannot support a decision.

Stop when one option remains robust across plausible ranges, or identify the
single affordable observation most likely to reverse the decision. Never hide
a preference inside invented precision or label a value judgment “falsified.”
