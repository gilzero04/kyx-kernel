# TDD: <Project Name>

project_id: <project_id>
author: <author>
created_by: <human|ai>
ai_prompt: "<full prompt>"
ai_confidence: 0.0-1.0
last_updated: YYYY-MM-DD

## 🧭 Reader Orientation (Rule 11)

- **Target Audience**: Developers and AI Agents.
- **Goal**: Define the Implementation Contract.

## Summary & Prime Directive (Rule 0)

**WHAT**: [Specific technical implementation details]
**WHY**: [Deep technical rationale]
**HOW**: [Code patterns, libraries, async model]

## Analysis & Decisions (Rule 4)

- **Algorithm/Logic**: [Description of core logic]
- **State Management**: [How state is preserved or passed]
- **Decision Record**: [Choice of specific crates/libraries/methods]

## Capability Traceability (Rule 5)

| Unit         | Purpose             | Infrastructure   | Source Signature |
| :----------- | :------------------ | :--------------- | :--------------- |
| [Func/Class] | [Atomic capability] | [Memory/Disk/DB] | [File::symbol]   |

## Invariants & Failure Modes (Rule 6)

- **Invariant**: [Types, bounds, concurrency safety]
- **Failure Mode**: [Serialization error, deadlock, panic]
- **Prevention**: [Result handling, sanitization, isolation]
