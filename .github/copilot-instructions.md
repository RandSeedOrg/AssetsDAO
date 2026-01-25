# GitHub Copilot Custom Instructions for this repository

Purpose
- Provide consistent, secure, and maintainable suggestions aligned with this repository's practices.
- Prefer clarity, small changes, and existing conventions over novelty.

Project context
- Private repository; treat all code/config/data as confidential.
- Monorepo structure with Cargo and npm workspaces.
  - Backend: Rust canisters in backend/ with shared libraries in backend/libraries/
  - Frontend: Multiple apps (ProjectL_frontend, admin-app, WLCommunity) in src/
- Match existing styles, patterns, naming, and architecture within each workspace.

Language policy
- Review comments: write in Simplified Chinese (简体中文) as the primary language, and add one concise English note line immediately after each Chinese point (prefix with "EN:").
- Pull request titles and descriptions: write in English (concise, action-oriented).
- Code identifiers, API names, and logs/errors: follow the repo's established language (typically English).

Review comment format
- Keep comments short and focused; prefer bullets.
- Example:
  - 中文：请将此函数拆分为更小的单元，避免在 await 期间持有可变状态。
    EN: Please split this function into smaller units and avoid holding mutable state across await.
  - 中文：此接口为只读，应标记为 query 并去除任何状态变更。
    EN: This endpoint is read-only; mark it as query and remove any state mutations.

General behavior
- Ask brief clarifying questions when requirements are ambiguous.
- Prefer minimal diffs and incremental improvements.
- Keep suggestions self-contained and runnable; include only essential imports/types.

ICP-specific guidance
- Canister interfaces
  - Maintain backward compatibility for Candid (.did). Avoid breaking changes; if necessary, include migration notes.
  - Use query for read-only deterministic calls; do not mutate state in query methods.
- State and upgrades
  - Persist state across upgrades using stable memory.
    - Rust: #[pre_upgrade]/#[post_upgrade]; consider ic-stable-structures for large/stable data.
    - Motoko: stable variables; implement preupgrade/postupgrade.
  - Version state and test upgrade/downgrade paths locally.
- Concurrency and await safety
  - Do not hold mutable state across await; follow "check -> effect -> interact".
  - Handle inter-canister call errors explicitly; inspect reject codes.
- Access control and identity
  - Validate caller (Rust: ic_cdk::caller(); Motoko: Principal/caller). Centralize authorization.
  - Avoid leaking principals or sensitive metadata in logs.
- Cycles and limits
  - Be explicit about cycles; estimate costs when adding inter-canister calls.
  - Respect message size limits (~2MB); chunk large payloads.
- Determinism, time, randomness
  - Use ic_cdk::api::time / Time.now(); do not rely on wall-clock guarantees for security.
  - For randomness, use raw_rand (update) and derive via DRBG for crypto needs.
  - Prefer integer math over floating point for critical logic.
- Testing on IC
  - Unit + canister-level tests (Rust: cargo test + pocket-ic/ic-repl; Motoko: moc + ic-repl).
  - Test upgrade hooks and inter-canister flows locally (dfx start).
- Logging
  - Use repo logging utilities or ic_cdk::println; redact sensitive data.

File size and structure policy
- Prefer keeping source files under 300 lines of code when practical.
  - For existing large files (>300 lines), avoid making them larger.
  - When creating new files, target <200 lines as a goal.
  - Split large files into focused modules when refactoring.
  - Prefer smaller functions and cohesive types; extract helpers.
  - For tests, favor many short cases over monolithic suites.
  - When touching a large file, consider if refactoring would help, but prioritize the immediate task.

Security and privacy
- Never include secrets, tokens, keys, or private URLs in code, comments, or tests.
- Environment-specific configuration files (.env.dev, .env.prod, .env.test) are tracked and contain canister IDs (public identifiers, not secrets).
- Use .env.local (gitignored) for any local development secrets or overrides.
- Validate inputs, sanitize external data, least privilege, and fail closed.
- Redact sensitive info in logs and errors; prefer structured logging if available.

Style and quality
- Match the repo's formatter and linter configurations:
  - Rust: Use rustfmt (defaults) and clippy for backend canisters
  - TypeScript/React: Use ESLint (eslint.config.js) and Prettier (npm run format)
  - Motoko: Follow consistent formatting conventions
  - All: Respect .editorconfig settings (2-space indentation, LF line endings)
- Prefer existing dependencies; add new ones only with clear justification (size, maintenance, security).
- Suggestions should pass lint/format checks and CI when applicable.

### 简化代码（可读性与维护性）

- 中文：尽量建议使用早返回、拆分长函数和提取重复块（循环、`try/catch`、数据转换），在不改变行为的前提下简化控制流。
  EN: Encourage early returns, splitting long functions, and extracting repeated blocks (loops, `try/catch`, transformations) without changing behavior.

- 中文：当发现与已有工具函数或 hook 功能重叠时，应优先建议复用现有实现，而不是新增一套类似逻辑。
  EN: When new code overlaps with existing utilities or hooks, suggest reusing the existing implementation instead of adding a parallel one.

- 中文：对于超过 300 行的文件和职责过多的组件，建议在本次改动范围内适度拆分（如展示组件与逻辑组件分离），避免进一步膨胀。
  EN: For files over 300 lines or multi-responsibility components, recommend reasonable splitting within the scope of the change to avoid further growth.

### 前后端函数使用一致性

- 中文：检查前端对 canister 方法的调用是否选择了合适的接口（如 `get_*_details` vs `get_*_summary`，公开 vs 私有数据），避免过度暴露或冗余数据。
  EN: Verify frontend calls use the appropriate canister methods (e.g., `get_*_details` vs `get_*_summary`, public vs private data) to avoid unnecessary exposure or payload.

- 中文：对于命名相近或职责相似的函数，核对每个调用点的业务需求是否匹配（只读 query vs 更新 update），并在不一致时给出更合适的选择建议。
  EN: For similarly named or scoped functions, confirm each call site matches the intended behavior (read-only query vs state-changing update) and suggest corrections when misused.

- 中文：注意前端类型与 Candid/Rust 定义是否对齐（可选字段、错误枚举、时间/数值单位等），发现前端对可选值或变体假定为必然存在时，应提醒增加安全检查。
  EN: Check that frontend types align with Candid/Rust definitions (optionals, error variants, time/number units), and flag places where optional/variant data is treated as always present.

- 中文：如果同一个后端方法在不同前端页面被包装成多套 service 或 hook，建议合并为单一封装，减少调用路径和维护成本。
  EN: When the same backend method is wrapped by multiple services/hooks in different pages, recommend consolidating into a single shared wrapper.

### 建议评论示例（统一格式）

- 中文：这里的逻辑与上面函数 `X` 基本相同，建议抽取成公共辅助函数以减少重复。
  EN: This logic is almost the same as function `X` above; consider extracting a shared helper to reduce duplication.

- 中文：该组件对同一 canister 方法维护了两套调用方式（hook + service），建议统一为一种以降低复杂度。
  EN: This component maintains both hook- and service-based calls for the same canister method; consider standardizing on a single approach.

- 中文：当前使用的是 `get_*_details`，但只需要摘要信息，建议改为 `get_*_summary` 以减少数据传输和暴露。
  EN: The code uses `get_*_details` though only summary data is needed; consider switching to `get_*_summary` to reduce data transfer and exposure.

Testing
- Update/add tests when changing logic. Keep them deterministic (no real network/time unless mocked).
- Run tests before committing:
  - Rust backend: `cargo test` (or `cargo test -p <package_name>` for specific workspace)
  - Frontend: `npm test` (workspace-specific or root-level)
  - Integration: Follow project-specific test documentation
- Use existing testing stacks and ensure they run under CI.

Documentation
- Update inline docs and README/MD when changing public APIs or behavior.
- Explain "why" and notable trade-offs; keep comments concise.

Git and PR workflow
- Small, focused commits with clear messages (present tense).
- Separate refactors/moves from logic changes where possible.
- Include migration notes (state layout, Candid changes, upgrade steps) in PR descriptions for any breaking or upgrade-relevant changes.

Checklist
- Chinese review comments + one-line English note
- English PR titles/descriptions
- Follows ICP-safe patterns (stable state, upgrade hooks, await safety, caller checks)
- Files kept reasonably sized; minimal diff; idiomatic style
- Passes format/lint/CI; tests added/updated
- No secrets in code; uses proper env/config patterns; concise, redacted logs

## Code review focus: redundancy and usage consistency

When reviewing code in this repository, pay special attention to redundant logic and correct usage of similar functions, especially across backend canisters and multiple frontend apps.

### Redundant or duplicate code

- 中文：在同一文件或相邻模块中，留意是否存在逻辑高度相似的代码块（校验、映射、错误处理、请求封装等），优先建议抽取为公共函数或工具。
  EN: In the same file or nearby modules, watch for highly similar logic (validation, mapping, error handling, request wrappers) and suggest extracting shared helpers.

- 中文：对于在多个 canister 或库中重复出现的逻辑（例如 `validate_*`、`normalize_*`、DTO 转换），优先建议迁移到 `backend/libraries/` 或已有共享库中。
  EN: For logic duplicated across canisters or libraries (e.g., `validate_*`, `normalize_*`, DTO conversions), recommend moving it into `backend/libraries/` or an existing shared library.

- 中文：前端多个 app（`ProjectL_frontend`、`admin-app`、`WLCommunity`）中若出现相似的 hooks、服务封装或组件结构，应建议抽取为共享 hook 或组件，避免拷贝粘贴。
  EN: If similar hooks, services, or component structures appear across multiple frontend apps, suggest extracting shared hooks or components instead of copy–paste.

- 中文：大函数或大组件中若存在重复分支逻辑，优先建议通过参数化、策略函数或小组件拆分来消除重复。
  EN: For large functions/components with repeated branch logic, recommend parameterization, strategy functions, or splitting into smaller components to remove duplication.

