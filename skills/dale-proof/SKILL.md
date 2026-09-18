---
name: dale-proof
description: Verify a concrete technical claim against direct, fresh evidence. Use for implementation, migration, deployment, runtime, or compatibility claims.
---

# Dale Proof

Test a claim at its stated boundary and return `PROVEN`, `PARTIAL`, `UNPROVEN`,
or `CONTRADICTED`. Verification is read-only unless the user separately asks
for remediation.

## Boundary and authority

- Preserve material quantifiers, target, revision, environment, time window,
  side effects, and forbidden outcomes. Split the claim into atomic obligations.
- Use the shortest evidence path that can observe each obligation. Source can
  prove implementation; tests can prove exercised cases; only matching runtime
  evidence can prove deployment or user-visible behavior.
- Treat absent, inaccessible, stale, or indirect evidence as `UNPROVEN`, not as
  success or contradiction. Direct falsification is `CONTRADICTED`; mixed or
  incomplete coverage is `PARTIAL`.
- Do not edit code, redeploy, restart, mutate data, trigger production side
  effects, or broaden permissions merely to obtain proof. Stop at the authority
  boundary and identify the missing signal.
- Redact secrets, credentials, private payloads, raw environment values, and
  unnecessary personal data. Do not run an unsafe reproduction.

Read [the detailed proof contract](references/proof-contract.md) only when the
claim needs its domain patterns, evidence-quality rubric, contradiction method,
or full proof-capsule schema. For a narrow claim, the rules above are sufficient.

## Verify

1. Restate the original claim and exact evidence boundary. Resolve cheap,
   non-material ambiguity from context; ask only when different answers would
   change the verdict.
2. Map each atomic obligation to its primary signal and the smallest safe check.
   Record target identity, revision, time, command or source, result, and
   coverage. A command name or green exit alone is not the observation.
3. Reconcile conflicts by directness, target fit, freshness, reproducibility,
   coverage, and source integrity. Keep unresolved version or boundary drift
   explicit.
4. Assign atomic verdicts, then compose the overall verdict deterministically:
   any material `CONTRADICTED` makes the whole claim `CONTRADICTED`; all
   material obligations must be `PROVEN` for overall `PROVEN`; a non-contradictory
   mix with at least one `PROVEN` or `PARTIAL` is `PARTIAL`; an all-`UNPROVEN`
   set is `UNPROVEN`.

## Return the proof capsule

Lead with the overall verdict and verified boundary. Include the atomic claim
table, direct primary signal, supporting evidence, failed or unavailable
checks, contradictions, freshness and coverage limits, and at most one next
decisive check when it could change the verdict.

Never let code inspection imply execution, a test imply deployment, a healthy
producer imply downstream receipt, or lack of observed failures imply universal
absence. If the requested boundary cannot be fully observed, return `PARTIAL`
or `UNPROVEN` according to the atomic coverage.
