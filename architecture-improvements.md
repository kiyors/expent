# Architectural & Structural Improvements Plan

## Background & Motivation
The current codebase has several architectural bottlenecks:
1. **OCR Orchestration Complexity:** `crates/expent_core/src/bridge.rs` uses a massive conditional block to process different document types, violating the Open/Closed Principle.
2. **Background Task Limitations:** The `bulk_confirm` API endpoint bounds concurrency but still holds the HTTP request open until all jobs complete, risking timeouts and poor UX. Current worker logic is also tightly coupled to the `ocr` crate.
3. **Database Read Overhead:** Static and highly-reused entity data (Wallets, Categories, Contacts) are queried repeatedly in loops during enrichment processes.
4. **Frontend Type Safety:** There are 25+ usages of `any` remaining in the dashboard, weakening TypeScript's safety guarantees.

## Scope & Impact
This plan covers deep refactoring of the backend orchestration, introduction of a new workspace crate for background jobs, application-level caching, and strict type enforcement on the frontend. The impact will be vastly improved modularity, reduced database load, non-blocking API responses, and fewer frontend runtime errors.

## Proposed Solution

### Phase 1: Generic Background Jobs Crate (`crates/jobs`)
Create a robust, generic, database-backed job queue to decouple long-running tasks from HTTP request lifecycles.
*   **Infrastructure:**
    *   Create a new SeaORM migration to define a `background_jobs` table (`id`, `job_type`, `payload` (JSON), `status`, `attempts`, `run_at`, `created_at`, `completed_at`, `error`).
    *   Initialize a new workspace crate `crates/jobs`.
*   **Implementation:**
    *   Implement a `JobQueue` trait and a `DbJobQueue` struct for enqueuing jobs.
    *   Implement a `WorkerPool` that spawns Tokio tasks to poll and execute jobs concurrently using `Semaphore` for rate-limiting.
    *   Implement a `JobHandler` trait for specific task logic.
*   **Integration:**
    *   Refactor the `bulk-confirm` API route to enqueue `BulkConfirmJob` payloads and return `202 Accepted` immediately.
    *   Update the dashboard to rely on SSE (Server-Sent Events) or polling to track bulk action progress.

### Phase 2: Application-Level Caching (`moka`)
Introduce high-performance, concurrent, in-memory caching for frequently accessed, slowly changing data.
*   **Implementation:**
    *   Add the `moka` crate to the workspace.
    *   Update `WalletsManager`, `CategoriesManager`, and `ContactsManager` in their respective crates to use `moka::future::Cache`.
    *   Implement cache invalidation logic upon `create`, `update`, and `delete` operations.
*   **Impact:** Drop database lookups for these entities during heavy operations like bank statement processing to near-zero.

### Phase 3: Strategy Pattern for OCR (`crates/expent_core/src/bridge.rs`)
Refactor the monolithic OCR processing logic.
*   **Implementation:**
    *   Define an `OcrExtractionStrategy` trait with an `extract_and_save` method.
    *   Extract the existing `BANK_STATEMENT`, `GPAY`, and generic logic into distinct strategy structs (`BankStatementStrategy`, `GPayStrategy`, `GenericStrategy`).
    *   Implement a factory or resolver to instantiate the correct strategy based on the document type.

### Phase 4: Strict Type Enforcement (Frontend)
Eliminate remaining `: any` usages in the dashboard.
*   **Implementation:**
    *   Systematically replace `any` with specific types from `@expent/types` in `shared-ledgers`, `reconciliation`, and `subscriptions` pages.
    *   Implement Zod schemas at the API boundary (using `api-client.ts`) where necessary to parse unknown JSON responses into typed data safely.

## Verification
*   **Backend:** Write robust unit tests for the new `crates/jobs` logic (enqueueing, dequeuing, retry mechanism). Verify OCR refactor using the existing `core_init_tests` and potentially new strategy-specific unit tests.
*   **Frontend:** Run `pnpm tsc` and `pnpm exec biome check` to guarantee 0 errors. Ensure the dashboard builds successfully.
*   **Integration:** Manually test a bulk OCR confirmation flow to verify the async job queue and frontend progress indication.

## Migration & Rollback
*   The `background_jobs` table addition is non-destructive.
*   If the new job queue introduces issues, we can revert the API endpoints to execute synchronously.
*   The OCR Strategy refactor is a purely structural change and relies on existing tests to prevent regressions.