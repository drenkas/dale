# TDD.md

## Meta

- Last updated:
- Owner:
- Status: draft | active | enforced
- Evidence baseline:

## 1. Test strategy

- Primary approach:
- Highest-value behavior:
- Test boundaries:

## 2. Test environments

- Local:
- CI:
- Staging or production checks:

## 3. Test inventory

| Type | Location | Command | Protects | Evidence |
|---|---|---|---|---|

## 4. Behavior workflow

1. Capture the user-visible or contract-level failure.
2. Make the smallest owning-layer change.
3. Prove the primary behavior.
4. Run coupled-layer and regression checks.
5. Refactor only while evidence remains green.

## 5. Requirement coverage

| Requirement | Test or check | Status | Gap |
|---|---|---|---|

## 6. Quality gates

| Gate | Exact command | Required result | Evidence |
|---|---|---|---|

## 7. Fixtures and test data

- Fixtures:
- Factories:
- Mocks or fakes:
- Sensitive-data constraints:

## 8. Flakiness and failure protocol

- Known flaky checks:
- Retry or quarantine policy:
- Failure ownership:

## 9. Missing coverage

| Priority | Behavior | Risk | Smallest useful test |
|---|---|---|---|
