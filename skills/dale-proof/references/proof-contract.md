# Dale Proof contract

Proof means empirical verification within a target, revision, time, and coverage
boundary, not mathematical, cryptographic, or universal certainty.

## Verdicts

| Verdict | Condition |
|---|---|
| `PROVEN` | Direct evidence satisfies the scoped atomic claim without unresolved material contradiction. |
| `PARTIAL` | Direct evidence satisfies only part of the material scope or target. |
| `UNPROVEN` | The signal is missing, inaccessible, stale, mismatched, or indirect. |
| `CONTRADICTED` | Direct evidence falsifies a material obligation. |

Missing evidence is `UNPROVEN`; record acquisition blockers separately, without
a fifth verdict. Overall: any material contradiction → `CONTRADICTED`; all
material claims proven → `PROVEN`; otherwise any `PROVEN` or `PARTIAL` → `PARTIAL`;
all unproven → `UNPROVEN`.

Map every original quantifier, environment, side effect, user outcome, and
forbidden outcome to an atomic claim or uncovered obligation. A narrower test
cannot prove the broader claim.

## Evidence quality and ladder

Assess directness (behavior or prerequisite), target fit (environment, boundary,
revision, identity), freshness (time and drift), reproducibility (repeatability
or independent inspection), coverage (inputs, states, failures, consumers, side
effects), and integrity (complete, sufficiently unaltered, no leaked secrets).
Resolve conflicts by these axes and source proximity, not majority vote.
Different evidence levels may truthfully describe different boundaries.

Use only the levels needed by the claim:

1. **Declared:** requirements, docs, schemas, configuration, migration intent.
2. **Implemented:** owner, producer/consumer, reads/writes, guards, transitions.
3. **Exercised:** targeted test or isolated reproduction with a real oracle.
4. **Integrated:** both contract sides or a realistic dependent surface.
5. **Observed:** requested runtime, serving deployment, user-visible outcome.

A branch's existence does not prove production execution. Unit tests may prove
a pure function within tested inputs; browser smoke does not prove backend effects.

## Claim patterns

- **Code fix:** failing case, successful behavior, forbidden side effect,
  neighboring path, regression boundary. Prefer a pre-fix witness or existing
  regression test plus a current targeted run; diffs/test names only support proof.
- **Deployment:** distinguish build, artifact revision, deployment selection,
  serving instance, traffic routing, endpoint behavior, user outcome. Prove only
  observed stages; never redeploy/restart solely for verification.
- **Scheduled/async:** registration, timezone, predicate, enqueue, delivery,
  recipient/consumer acknowledgement, retries, duplicate suppression.
  Scheduler health proves neither delivery nor receipt.
- **Shared contract:** separate declared/implemented/exercised/observed;
  check producer/consumer, success/errors, defaults/omission, version skew,
  environment/revision differences. Preserve drift rather than choosing one
  globally authoritative source.
- **Absence/universality:** define search universe, interval, identity scheme,
  detector capability. No observed failure proves absence only within known
  detector coverage; otherwise use `PARTIAL` or `UNPROVEN`.

## Contradictions and capsule

For each conflict record exact propositions, source target/revision/time,
evidence level, and the smallest safe check distinguishing version drift,
boundary difference, stale evidence, or real inconsistency. Do not hide atomic
contradictions behind an overall verdict.

Return original claim and quantifiers, verified scope/identity, atomic verdicts,
primary signal and exact result, supporting evidence by level, failed/blocked/
skipped/unavailable checks, unresolved conflicts, freshness/coverage limits,
a safe reproduction when available, and at most one next check that could
change the verdict.

Exclude secrets, credentials, raw private payloads, and unrestricted logs.
Use redacted identifiers and only the sensitive evidence needed for the verdict.
