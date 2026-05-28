# Agent Instructions

## Tech Stack & Structure

- **Frontend**: `apps/dashboard` (Next.js), `apps/app` (Expo)
- **Backend**: `apps/api` (Rust Axum)
- **Central Hub**: `crates/expent_core` (Orchestrates DB, Auth, Upload, OCR)
- **Shared**: `packages/types` (Shared TS/Rust types), `packages/ui` (Shared UI)
- **Testing**: `rstest` (Rust Backend & API), `vitest` (Next.js Headless Logic). No UI E2E.

## Package Managers

- **JS/TS**: **pnpm**: `pnpm install`, `pnpm dev`, `pnpm add <package name> --filter`, `pnpm dlx shadcn@latest add button -c apps/dashboard`, `pnpm fmt`
- **Rust**: **cargo**: `cargo check`, `cargo run -p api`

## File-Scoped Commands

| Task            | Command                                   |
| --------------- | ----------------------------------------- |
| Typecheck       | `pnpm tsc --noEmit path/to/file.ts`       |
| Lint (JS)       | `pnpm lint`                               |
| Format (JS)     | `pnpm fmt`                                |
| Lint (Rust)     | `cargo clippy --fix -p <crate> -- <file>` |
| Format (Rust)   | `cargo fmt`                               |
| Test (Rust)     | `cargo test -p expent_core --lib`         |
| Test (JS/TS)    | `pnpm vitest run path/to/file.test.ts`    |
| Format (All)    | `pnpm fmt-all`                            |

## Documentation

- `AGENTS.md`: Canonical agent-facing documentation. Keep under 80 lines.
- `GEMINI.md`: Foundational mandates for Gemini CLI specifically.
- `docs/core.md`: Deep dive into the Centralized Hub Architecture.

# Skill mappings - when working in these areas, load the linked skill file into context.

skills:

- task: "Managing local database collections and live queries"
  load: "node_modules/@tanstack/react-db/skills/react-db/SKILL.md"
- task: "Setting up typed collections and selecting sync adapters"
  load: "node_modules/@tanstack/db/skills/db-core/collection-setup/SKILL.md"
- task: "Implementing optimistic mutations and transactions"
  load: "node_modules/@tanstack/db/skills/db-core/mutations-optimistic/SKILL.md"
- task: "Working with persistent local storage (WA-SQLite, expo-sqlite)"
  load: "node_modules/@tanstack/db/skills/db-core/persistence/SKILL.md"
- task: "Configuring environment variables and secrets"
  load: "node_modules/dotenv/skills/dotenvx/SKILL.md"
- task: "Implementing authentication, two-factor, or organization best practices"
  load: "better-auth/skills"
- task: "Building or debugging Expo native UI, API routes, and deployments"
  load: "expo/skills"
- task: "Managing Next.js frontend, Vercel deployments, and React best practices"
  load: "vercel-labs/agent-skills"
- task: "Frontend UI, UX polish, and building web components"
  load: "anthropics/skills"
- task: "Rust API best practices and architecture"
  load: "apollographql/skills"

## 🛠️ Active Skill Constraints

When generating code or reviewing PRs, you must actively apply the loaded skills:

### 1. Authentication & Security (`better-auth`)

- **Core:** Follow `better-auth-best-practices` and `create-auth-skill`.
- **Security:** Adhere to `better-auth-security-best-practices`.
- **Flows:** Use `email-and-password-best-practices` and `two-factor-authentication-best-practices`.
- **Multi-tenant:** Structure orgs according to `organization-best-practices`.

### 2. Next.js & Web Ecosystem (`vercel-labs`)

- **Architecture:** Use App Router conventions (`next-best-practices`).
- **React Patterns:** Write modern, concurrent React (`vercel-react-best-practices`).
- **Transitions:** Implement fluid navigation (`vercel-react-view-transitions`).

### 3. Expo & React Native Ecosystem

- **UI/UX:** Apply `building-native-ui` and `expo-tailwind-setup` for styling.
- **Architecture:** Utilize `expo-api-routes` and `native-data-fetching`.
- **Infrastructure:** Follow guidelines for `expo-cicd-workflows` and `expo-deployment`.

### 4. Frontend UI/UX Design

- **Components:** Build accessible interfaces using `shadcn` patterns.
- **Design:** Apply `frontend-design` and `web-design-guidelines`.
- **UX:** Polish using `ui-ux-pro-max` (micro-interactions, loading states).

### 5. Backend, Database & OCR

- **Rust Core:** TDD Red-Green-Refactor is mandatory. Use `#[rstest]` fixtures (`rust-best-practices`).
- **Transactions:** Atomic operations MUST use `db.transaction`. Always adjust wallet balances.
- **Rust OCR:** Background job processing natively integrated in `crates/ocr` using Postgres LISTEN/NOTIFY and graceful shutdown tokens.
- **Database:** Pure entities go in `crates/db/src/entities/`. No business logic here. Design via `database-schema-designer`.

### 6. Repository Architecture (Monorepo Boundaries)

- **Central Hub (`crates/expent_core`):** Orchestrates business rules, auth, and OCR delegation.
  - _Current:_ logic lives in the domain crates (`crates/wallets`, `crates/transactions`, `crates/ocr`, …) and is surfaced through the `expent_core` facade as `expent_core::<domain>` (e.g. `expent_core::ocr`, `expent_core::wallets`).
  - _Target:_ consolidate this logic under `expent_core/src/services/`, split into granular files. New cross-crate orchestration should move toward this layout.
- **API Entry (`apps/api`):** API routes strictly delegate to the `expent_core` facade (`expent_core::<domain>`; target: `expent_core::services`). No business logic in routes.
- **Shared Packages:** Do not define types or UI locally within apps. Use `packages/types` (generated via `ts-rs`) and `packages/ui`.
- **Dependency Management:** Common Rust dependencies belong in root `Cargo.toml` using `workspace = true`.

## 📝 General Directives

1. **Refactoring:** Consider UI/UX, database transaction safety, and Rust compiler guarantees first.
2. **Code Style:** Keep functions pure, use declarative patterns, prioritize readability.
3. **Documentation:** Comment on complex server-side logic, OCR processing steps, and Rust trait implementations.
