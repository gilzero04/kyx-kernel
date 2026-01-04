-- ============================================================
-- Kyx Governance: Lint Management Rule
-- Created: 2026-01-04
-- Priority: 85 (High - Code Quality Foundation)
-- ============================================================

-- Insert Lint Management Rule
UPDATE mcp_rules SET
    title = "Lint Management & Code Quality",
    content = "Rule: Lint Management & Code Quality (MANDATORY).

## 1. Zero Warning Policy
ALL projects in the Kyx Ecosystem MUST maintain a 'Zero Warning' state for both linting and type checking before committing code.

- **ESLint**: No errors, no warnings
- **Type Checker** (svelte-check/tsc): No errors, no warnings
- **Formatter** (Prettier): Code must be formatted

## 2. Pre-Development Planning
Before starting ANY development task:
1. **Check Current Lint Status**: Run `bun run lint` and `bun run check`
2. **Document Baseline**: Record current error/warning count
3. **Plan Lint Strategy**: 
   - Will new code introduce warnings?
   - How will you maintain zero-warning state?
   - What auto-fix rules can be applied?

## 3. During Development
- **Incremental Fixes**: Fix lint issues as you code, don't accumulate debt
- **Auto-Fix First**: Use `eslint --fix` and `prettier --write` before manual fixes
- **Type Safety**: Prefer proper types over `any` or `unknown`
- **Test Frequently**: Run lint checks after each significant change

## 4. Authorized Auto-Run Commands
AI agents are authorized to run these commands with `SafeToAutoRun: true`:
- `bun run lint`
- `bun run check`
- `bun run format`
- `npx eslint . --fix`
- `svelte-kit sync`
- `bun install` (when needed for linting)

## 5. Lint Debt Management
If inheriting a project with existing lint issues:
1. **Tier 1**: Configuration tuning (reduce noise)
2. **Tier 2**: Bulk systemic fixes (auto-fixable issues)
3. **Tier 3**: Core refactoring (manual type safety improvements)

## 6. Commit Rules
- **NEVER commit** code with new lint errors
- **NEVER commit** code with new type errors
- **ALWAYS verify** `bun run lint` and `bun run check` pass before commit

## 7. Incident Reporting
If lint issues block development:
- Use `report-incident()` to document the blocker
- Include: error count, affected files, proposed solution
- Update incident when resolved with `update-incident()`

## Rationale
Lint warnings are early indicators of potential runtime errors, type mismatches, and code quality issues. Addressing them proactively prevents technical debt and improves maintainability.",
    priority = 85,
    project_id = mcp_projects:global,
    type = "global",
    updated_at = time::now()
WHERE title = "Lint Management & Code Quality";

-- If rule doesn't exist, create it
IF (SELECT count() FROM mcp_rules WHERE title = "Lint Management & Code Quality") = 0 THEN
    CREATE mcp_rules CONTENT {
        title: "Lint Management & Code Quality",
        content: "Rule: Lint Management & Code Quality (MANDATORY).

## 1. Zero Warning Policy
ALL projects in the Kyx Ecosystem MUST maintain a 'Zero Warning' state for both linting and type checking before committing code.

- **ESLint**: No errors, no warnings
- **Type Checker** (svelte-check/tsc): No errors, no warnings
- **Formatter** (Prettier): Code must be formatted

## 2. Pre-Development Planning
Before starting ANY development task:
1. **Check Current Lint Status**: Run `bun run lint` and `bun run check`
2. **Document Baseline**: Record current error/warning count
3. **Plan Lint Strategy**: 
   - Will new code introduce warnings?
   - How will you maintain zero-warning state?
   - What auto-fix rules can be applied?

## 3. During Development
- **Incremental Fixes**: Fix lint issues as you code, don't accumulate debt
- **Auto-Fix First**: Use `eslint --fix` and `prettier --write` before manual fixes
- **Type Safety**: Prefer proper types over `any` or `unknown`
- **Test Frequently**: Run lint checks after each significant change

## 4. Authorized Auto-Run Commands
AI agents are authorized to run these commands with `SafeToAutoRun: true`:
- `bun run lint`
- `bun run check`
- `bun run format`
- `npx eslint . --fix`
- `svelte-kit sync`
- `bun install` (when needed for linting)

## 5. Lint Debt Management
If inheriting a project with existing lint issues:
1. **Tier 1**: Configuration tuning (reduce noise)
2. **Tier 2**: Bulk systemic fixes (auto-fixable issues)
3. **Tier 3**: Core refactoring (manual type safety improvements)

## 6. Commit Rules
- **NEVER commit** code with new lint errors
- **NEVER commit** code with new type errors
- **ALWAYS verify** `bun run lint` and `bun run check` pass before commit

## 7. Incident Reporting
If lint issues block development:
- Use `report-incident()` to document the blocker
- Include: error count, affected files, proposed solution
- Update incident when resolved with `update-incident()`

## Rationale
Lint warnings are early indicators of potential runtime errors, type mismatches, and code quality issues. Addressing them proactively prevents technical debt and improves maintainability.",
        priority: 85,
        project_id: mcp_projects:global,
        type: "global",
        created_at: time::now(),
        updated_at: time::now()
    };
END;
