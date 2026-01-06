# TEST_PLAN: <Project Name>

project_id: <project_id>
author: <author>
created_by: <human|ai>
ai_prompt: "<full prompt>"
ai_confidence: 0.0-1.0
last_updated: YYYY-MM-DD

## 🧭 Reader Orientation

- **Target Audience**: QA and Developers.
- **Goal**: Define the Verification Contract.

## Summary & Prime Directive

**WHAT**: [Testing strategy]
**WHY**: [Quality assurance goals]
**HOW**: [Unit, Integration, E2E]

## Analysis & Decisions

- **Test Selection**: [Why we test X and ignore Y]
- **Decision Record**: [Tools chosen for testing]

## Capability Traceability

| Scenario | Requirement      | Evidence Path |
| :------- | :--------------- | :------------ |
| [TS-01]  | [PRD Capability] | [tests/*]     |

## Invariants & Failure Modes

- **Invariant**: [Isolations must be maintained]
- **Failure Mode**: [Test flake, false positive]
- **Prevention**: [Parallelization strategies, deterministic seeds]
