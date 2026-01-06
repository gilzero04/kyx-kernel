# DATABASE_SCHEMA: <Project Name>

project_id: <project_id>
author: <author>
created_by: <human|ai>
ai_prompt: "<full prompt>"
ai_confidence: 0.0-1.0
last_updated: YYYY-MM-DD

## 🧭 Reader Orientation (Rule 11)

- **Target Audience**: Data Engineers.
- **Goal**: Define the Persistence Contract.

## Summary & Prime Directive (Rule 0)

**WHAT**: [Schema overview - tables, indices, relations]
**WHY**: [Why this structure supports the PRD/SAD]
**HOW**: [SQL, SurrealQL, Graph links]

## Analysis & Decisions (Rule 4)

- **Modeling Strategy**: [Normalization vs Denormalization]
- **Indexing Strategy**: [How we optimize for search]

## Capability Traceability (Rule 5)

| Table   | Relation    | Capability Mapping | Source Signature       |
| :------ | :---------- | :----------------- | :--------------------- |
| [Table] | [Link/Edge] | [PRD Requirement]  | [migration_file.surql] |

## Table Definitions (Rule 6)

### 1. [Table Name]

- `field`: type (description)

## Invariants & Failure Modes (Rule 6)

- **Invariant**: [Unique constraints, required fields]
- **Failure Mode**: [Integrity violation, slow query]
- **Prevention**: [Indices, foreign keys, validation triggers]
