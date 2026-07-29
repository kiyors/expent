# Expent

Expent is an intelligent expense management platform built with Rust and TypeScript. It features OCR-based receipt ingestion, automated subscription detection, and shared ledgers for group expense tracking.

## Architecture

- **`apps/api` (Rust/Axum):** Lean entry point and HTTP routing layer.
- **`crates/expent_core` (Rust):** The centralized logic hub ("Bank Brain") orchestrating database, auth, storage, and OCR services.
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

- **Centralized Core**: All business logic is strictly decoupled from the API and DB layers within `expent_core`.
- **Smart Merge**: Automatically deduplicates transactions by matching OCR results with existing bank records.
- **Itemized Splits**: Automatically parse receipt line items and split them across shared ledgers.
- **Subscription Engine**: Detects recurring payment patterns and alerts users of upcoming renewals.
- **Local-First Speed**: Instant dashboard rendering powered by OPFS and `wa-sqlite` caching on the frontend.
- **One-Click Demo Data**: Instantly provision a realistic mock environment (wallets, budgets, and transactions) to test drive the platform without manual data entry.
- **Group Ledgers**: Collaborative spaces for tracking expenses with friends and family.

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
