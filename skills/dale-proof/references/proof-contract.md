# Dale Proof contract

Use this contract to keep verdicts scoped and evidence-bound. “Proof” here means
empirical verification within an explicit target, revision, time, and coverage
boundary; it is not a claim of mathematical, cryptographic, or universal
certainty.

## Verdicts

| Verdict | Required condition |
|---|---|
| `PROVEN` | Direct evidence satisfies the atomic claim inside the stated scope, with no unresolved material contradiction. |
| `PARTIAL` | Direct evidence satisfies only part of the material scope or target. |
| `UNPROVEN` | The required signal is missing, inaccessible, stale, mismatched, or too indirect. |
| `CONTRADICTED` | Direct evidence falsifies a material obligation. |

Use `UNPROVEN`, not `CONTRADICTED`, when evidence is merely absent. Record an
acquisition blocker separately; do not invent a fifth claim verdict.

Compose an overall verdict in this order: any material `CONTRADICTED` claim
makes the overall claim `CONTRADICTED`; all material claims must be `PROVEN` for
overall `PROVEN`; a mix containing at least one `PROVEN` or `PARTIAL` claim and
no contradiction is `PARTIAL`; an all-`UNPROVEN` set is `UNPROVEN`.

Preserve the original claim and map every material quantifier, environment,
side effect, user outcome, and forbidden outcome to an atomic claim or an
explicit uncovered obligation. A narrowed test scope can support its atomic
claim, but it cannot make the broader original claim `PROVEN`.

## Evidence quality

Judge every material observation on these axes:

| Axis | Question |
|---|---|
| directness | Does it observe the claimed behavior or only a prerequisite? |
| target fit | Is it from the requested environment, boundary, revision, and identity? |
| freshness | Was it observed in the relevant time window, and can it have drifted? |
| reproducibility | Can the observation be repeated or independently inspected? |
| coverage | Which inputs, states, failure paths, consumers, and side effects were exercised? |
| integrity | Is the source complete, unaltered enough for the claim, and free of leaked secrets? |

Resolve conflicts by these axes and source proximity, not majority vote. A
formal declaration, implementation, test, runtime capture, and user-visible
observation may each truthfully describe a different boundary.

## Evidence ladder

Use the shortest sufficient path; do not run every level ceremonially.

1. **Declared** — requirements, docs, schemas, configuration, migration intent.
2. **Implemented** — owning code path, producer and consumer, read and write
   behavior, guards, state transitions.
3. **Exercised** — targeted test or isolated reproduction with a real oracle.
4. **Integrated** — both sides of a contract or realistic dependent surface.
5. **Observed** — requested runtime, serving deployment, or user-visible outcome.

A higher level is necessary only when the claim names that level. Code can prove
that a branch exists; it cannot prove the branch ran in production. A targeted
unit test may fully prove a pure function for its tested domain; a browser smoke
test may not prove backend side effects.

## Common claim patterns

### Code fix

Split “fixed” into the original failing case, expected successful behavior,
forbidden side effect, relevant neighboring path, and regression boundary.
Prefer a pre-fix witness or existing regression test plus a current targeted
run. A diff or test name alone is supporting evidence.

### Deployment

Distinguish build success, artifact revision, selected deployment, serving
instance identity, traffic routing, endpoint behavior, and user outcome. Prove
only the stages directly observed. Never redeploy or restart solely to verify.

### Scheduled or asynchronous behavior

Distinguish registration, timezone, selection predicate, enqueue, delivery,
recipient or consumer acknowledgement, retry, and duplicate suppression. A
healthy scheduler proves neither delivery nor receipt.

### Shared contract

Report declared, implemented, exercised, and observed contracts separately.
Check producer and consumer, success and error cases, defaults, omission,
version skew, and environment or revision differences. Preserve drift instead
of choosing one source as globally authoritative.

### Absence or universal claim

Define the search universe, interval, identity scheme, and detector capability.
“No failures observed” proves absence only inside the detector's known coverage.
Otherwise return `PARTIAL` or `UNPROVEN`.

## Contradictions

For each conflict, record:

1. the exact propositions in conflict;
2. source target, revision, and time;
3. whether each source is declared, implemented, exercised, integrated, or
   observed evidence;
4. the smallest safe check that would distinguish version drift, boundary
   difference, stale evidence, or a real inconsistency.

Do not erase a contradiction with a generic overall verdict. Verdicts belong to
atomic claims and explicit boundaries.

## Proof capsule requirements

A complete proof capsule contains:

- original claim with material quantifiers preserved;
- exact verified scope and target identity;
- material atomic claims with individual verdicts;
- direct primary signal and exact result;
- supporting evidence labeled by level;
- failed, blocked, skipped, and unavailable checks;
- unresolved contradictions;
- coverage and freshness boundary;
- safe reproduction path when available;
- at most one next decisive check, and only when it can change the verdict.

Never include secrets, raw private payloads, unrestricted logs, or credentials.
Summarize sensitive observations with redacted identifiers and the minimum
evidence needed to support the verdict.
