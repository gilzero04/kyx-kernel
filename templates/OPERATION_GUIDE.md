# OPERATION_GUIDE: <Project Name>

project_id: <project_id>
author: <author>
created_by: <human|ai>
ai_prompt: "<full prompt>"
ai_confidence: 0.0-1.0
last_updated: YYYY-MM-DD

## 🧭 Reader Orientation

- **Target Audience**: Operators and SRE.
- **Goal**: Define the Manual for operations.

## Summary & Prime Directive

**WHAT**: [Runbook summary]
**WHY**: [Uptime goals]
**HOW**: [Monitoring, logs, scaling]

## Analysis & Decisions

- **Maintenance Policy**: [Patch cycles]
- **Decision Record**: [Backup frequency]

## Capability Traceability

| Op      | Mechanism | Evidence       |
| :------ | :-------- | :------------- |
| [Scale] | [CLI/UI]  | [docker stats] |

## Invariants & Failure Modes

- **Invariant**: [System status always queryable]
- **Failure Mode**: [Log flood]
- **Prevention**: [Log rotation, alerting thresholds]
