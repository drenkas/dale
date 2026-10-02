---
name: dale-proof
description: Verify a concrete technical claim against direct, fresh evidence. Use for implementation, migration, deployment, runtime, or compatibility claims.
---

# Dale Proof

Verify a claim at its stated boundary. Stay read-only unless the user requests
remediation. Do not edit, redeploy, restart, mutate data, trigger production
side effects, or broaden permissions for proof. Stop at the authority boundary
and identify the missing signal. Avoid unsafe reproductions; redact secrets,
credentials, private payloads, raw environment values, and unnecessary personal data.

## Verify

1. Restate the claim, preserving material quantifiers, target, revision,
   environment, time window, side effects, and forbidden outcomes. Split it
   into atomic obligations. Resolve cheap ambiguity from context; ask only
   when answers would change the verdict.
2. Map each obligation to its primary signal and smallest safe check. Record
   target, revision, time, command/source, actual result, and coverage;
   a command name or green exit alone is not an observation.
3. Reconcile conflicts by directness, target fit, freshness, reproducibility,
   coverage, and source integrity. State unresolved version or boundary drift.
4. Assign atomic verdicts: `PROVEN` for direct satisfaction, `CONTRADICTED` for
   direct falsification, `PARTIAL` for incomplete coverage, `UNPROVEN` for absent,
   inaccessible, stale, or indirect evidence.
5. Compose the overall verdict: any material contradiction → `CONTRADICTED`;
   all material obligations proven → `PROVEN`; otherwise any `PROVEN` or
   `PARTIAL` → `PARTIAL`; all unproven → `UNPROVEN`.

Use the shortest sufficient evidence path. Source proves implementation, tests
prove exercised cases, and matching runtime evidence proves deployment or
user-visible behavior. Code inspection cannot imply execution, tests cannot
imply deployment, producer health cannot imply downstream receipt, and no
observed failures cannot imply universal absence.

Read [proof-contract.md](references/proof-contract.md) only for domain patterns,
the evidence rubric, contradiction method, or full capsule schema.

## Return

Lead with verdict and verified boundary. Include atomic claims/verdicts, primary
signal, supporting evidence, failed/unavailable checks, contradictions, freshness
and coverage limits, and at most one next check that could change the verdict.
An incompletely observed boundary remains `PARTIAL` or `UNPROVEN` according to
atomic coverage.
