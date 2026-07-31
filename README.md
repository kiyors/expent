# Tameio (ταμείο)

**Tameio (ταμείο)**: Derived from ancient Greek, it means the place where money is kept, counted, and exchanged. It acts as both a "till" for your everyday receipts and a "treasury" for shared funds and subscriptions.

Tameio is an intelligent expense management platform built with Rust and TypeScript. It is designed to eliminate the friction of personal and group finance by automating data entry through AI-powered OCR, intelligently reconciling bank data, and providing a local-first, lightning-fast user experience.

## Architecture

- **`apps/api` (Rust/Axum):** Lean entry point and HTTP routing layer.
- **`crates/tameio_core` (Rust):** The centralized logic hub ("Bank Brain") orchestrating database, auth, storage, and OCR services.
- **`apps/dashboard` (TanStack Start / React Router):** Modern React dashboard using TanStack Query, Vite, and Zustand.
- **`crates/ocr` (Rust):** Native background worker processing pipeline leveraging Gemini 2.5 Flash for deterministic JSON extraction.
- **`packages/ui`:** Shared component library built with Tailwind CSS and Shadcn.
- **`packages/types`:** Shared TypeScript types automatically generated from Rust models via `ts-rs`.

## Prerequisites

- **Node.js:** v24 or higher (pnpm recommended)
- **Rust:** Latest stable version
- **Database:** PostgreSQL (recommended)
- **Storage:** Cloudflare R2 or S3-compatible storage

## Getting Started

1. **Clone the repository**
2. **Install dependencies:**
   ```bash
   pnpm install
   ```
3. **Configure environment variables:**
   Copy `.env.example` to `.env` in the root and fill in your credentials.

   ```bash
   cp .env.example .env
   ```

4. **Initialize database and run migrations:**
   ```bash
   # From the project root
   cargo run -p migration -- up
   ```
5. **Start development server:**
   ```bash
   pnpm dev
   ```

## Key Features

- **Centralized Core**: All business logic is strictly decoupled from the API and DB layers within `tameio_core`.
- **Smart Merge**: Automatically deduplicates transactions by intelligently matching highly detailed OCR receipt data with cryptic bank records, linking physical receipts to digital transactions without manual intervention.
- **Itemized Splits**: The OCR pipeline doesn't just read the total; it parses individual line items, allowing users to split specific items (e.g., splitting a dinner bill item-by-item) across shared ledgers seamlessly.
- **Subscription Engine**: Detects recurring payment patterns and alerts users of upcoming renewals, helping identify forgotten subscriptions and manage cash flow.
- **Local-First Speed**: Instant dashboard rendering powered by OPFS and `wa-sqlite` caching on the frontend, ensuring navigation through thousands of transactions is completely instantaneous.
- **One-Click Demo Data**: Instantly provision a realistic mock environment (wallets, budgets, transactions, and contacts) to test-drive the platform without manual data entry.
- **Group Ledgers & P2P Tracking**: Collaborative spaces for tracking expenses with friends and family. Tracks who paid for what and calculates optimal settlement paths to clear debts.

## Environment Variables

| Variable               | Description                                        |
| :--------------------- | :------------------------------------------------- |
| `DATABASE_URL`         | PostgreSQL connection string                       |
| `AUTH_SECRET`          | 32+ character secret for authentication            |
| `S3_ENDPOINT`          | S3-compatible API endpoint                         |
| `S3_ACCESS_KEY_ID`     | Access key for storage                             |
| `S3_SECRET_ACCESS_KEY` | Secret key for storage                             |
| `S3_BUCKET_NAME`       | Name of the bucket for uploads                     |
| `GOOGLE_API_KEY`       | Google Gemini API Key for OCR                      |
| `GEMINI_MODEL`         | Gemini model version (e.g. `gemini-2.0-flash-exp`) |
