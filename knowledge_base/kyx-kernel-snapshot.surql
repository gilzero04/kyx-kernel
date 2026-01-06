-- Kyx Governance Snapshot: kyx-kernel
-- Generated at: 2026-01-06T19:20:59Z
-- Documents found: 8

USE NS kyx; USE DB governance;

BEGIN TRANSACTION;

-- 1. Documents
UPSERT mcp_documentation:3myb9l4z2ovsqq3d52pa CONTENT { name: 'api-spec', title: 'Kernel API Specification', sdlc_phase: 'design', mimeType: 'text/markdown', project_id: mcp_projects:kernel, content: '# Kernel API Spec\n\n## v1.0\nEndpoints for dynamic tool dispatching, user management, and system monitoring.' };
UPSERT mcp_documentation:cvsnovqmcol026h8r8qn CONTENT { name: 'architecture', title: 'Kernel System Architecture', sdlc_phase: 'design', mimeType: 'text/markdown', project_id: mcp_projects:kernel, content: '# Kyx Kernel Architecture\n\n## Layers\n1. **I/O Layer**: Ntex-based async handlers.\n2. **Security layer**: RBAC and JWT verification.\n3. **Logic Interpreter**: Executes business rules fetched from Metadata Hub.' };
UPSERT mcp_documentation:cg6fr5sd4eaiydoyha6u CONTENT { name: 'dynamic-ui-architecture', title: 'Dynamic Plugin UI Architecture', sdlc_phase: 'design', mimeType: 'text/markdown', project_id: mcp_projects:kernel };
UPSERT mcp_documentation:01959d7c4c537d4cb2e2 CONTENT { name: 'unified-asset-architecture', title: 'Unified Asset Architecture', sdlc_phase: 'design', mimeType: 'text/markdown', project_id: mcp_projects:kernel };
UPSERT mcp_documentation:e2e_granular_permissions CONTENT { name: 'e2e-tests-granular-permissions-fix', title: 'E2E Tests & Granular Permissions Fix', sdlc_phase: 'implementation', mimeType: 'text/markdown', project_id: mcp_projects:kernel };
UPSERT mcp_documentation:twejfy8bjmdoqasi3424 CONTENT { name: 'project-workflow-status', title: 'Master Workflow: Kyx Kernel', sdlc_phase: 'maintenance', mimeType: 'text/markdown', project_id: mcp_projects:kernel };
UPSERT mcp_documentation:ldkejg7w1stu90gcul0s CONTENT { name: 'prd', title: 'Kernel Product Requirement Document', sdlc_phase: 'planning', mimeType: 'text/markdown', project_id: mcp_projects:kernel };
UPSERT mcp_documentation:9zaze041llftgoatv4j4 CONTENT { name: 'test-plan', title: 'Kernel Test Plan', sdlc_phase: 'verification', mimeType: 'text/markdown', project_id: mcp_projects:kernel };

-- 2. Incidents
UPSERT mcp_incident:gjbdrqptciipgpgexg7u CONTENT { 
    title: 'E2E Tests Failures + Missing Granular Permissions', 
    status: 'solved', 
    programming_language: 'Rust', 
    symptom: '8 E2E tests failing: login 400→401, i18n 404, plugins 400/401, media assets 404, permissions 403, user_preferences 500',
    root_cause: '1. user_preferences 500: Missing id column in migration. 2. Plugins 400/401: Routes without auth middleware. 3. i18n 404: GET routes not registered. 4. Permissions 403: superadmin missing assignments. 5. Single-scope permissions violated RBAC.',
    solution: '1. Migration 0020 (user_preferences). 2. Plugin router with RequirePermission. 3. i18n GET routes. 4. Migration 0021 (superadmin). 5. Migration 0022 (12 granular permissions). 6. Refactored 3 routers.',
    project_id: mcp_projects:kernel 
};

COMMIT TRANSACTION;
