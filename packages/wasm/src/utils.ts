import type {
  AggregatedMetrics,
  BatchMatchResult,
  BudgetPeriod,
  DashboardSummary,
  DetectedSubscription,
  FuzzySearchResult,
  SavingsProjection,
  SearchableField,
  SearchableItem,
  SpendingVelocity,
  Txn,
  TxnPattern,
} from "@expent/types";

/**
 * Loads the wasm module.
 * Note: This must be called inside a useEffect or after a user interaction
 * because it is an async WebAssembly module.
 */
export async function loadExpentWasm() {
  const wasm = await import("../pkg/wasm");
  // Initialize the WASM module (required for --target web)
  await wasm.default();
  return wasm;
}

/**
 * Calculates budget percentage consumption using Rust/WASM.
 *
 * Returns `undefined` when either input is not a parseable Decimal
 * (previously this silently produced "0" — callers should treat
 * `undefined` as "bad input" rather than "zero").
 */
export async function calculateBudgetPercentageWasm(spent: string, limit: string): Promise<string | undefined> {
  const wasm = await loadExpentWasm();
  return wasm.calculate_budget_percentage(spent, limit);
}

/**
 * Checks if a transaction date is within a specific budget period using Rust/WASM.
 */
export async function isTransactionInPeriodWasm(txnDate: string | number | Date, period: BudgetPeriod) {
  const wasm = await loadExpentWasm();
  const date = new Date(txnDate);
  return wasm.is_transaction_in_period(BigInt(date.getTime()), period);
}

/**
 * Calculates spending velocity using Rust/WASM.
 */
export async function calculateSpendingVelocityWasm(spent: string, limit: string, period: string) {
  const wasm = await loadExpentWasm();
  return wasm.calculate_spending_velocity(spent, limit, period) as SpendingVelocity | undefined;
}

/**
 * Projects a savings goal timeline using Rust/WASM.
 */
export async function projectSavingsGoalWasm(
  currentBalance: string,
  targetAmount: string,
  monthlyIncome: string,
  monthlyExpenses: string,
) {
  const wasm = await loadExpentWasm();
  return wasm.project_savings_goal(currentBalance, targetAmount, monthlyIncome, monthlyExpenses) as
    | SavingsProjection
    | undefined;
}

/**
 * Normalizes text for comparison using Rust/WASM.
 */
export async function normalizeTextWasm(text: string) {
  const wasm = await loadExpentWasm();
  return wasm.normalize_text(text);
}

/**
 * Generates a phonetic representation of text using Rust/WASM.
 */
export async function phoneticEncodeWasm(text: string) {
  const wasm = await loadExpentWasm();
  return wasm.phonetic_encode(text);
}

/**
 * Calculates a Jaro-Winkler fuzzy score between two strings using Rust/WASM.
 */
export async function fuzzyScoreWasm(a: string, b: string) {
  const wasm = await loadExpentWasm();
  return wasm.fuzzy_score(a, b);
}

/**
 * Calculates a match score between a statement row and a transaction using Rust/WASM.
 */
export async function calculateMatchScoreWasm(
  rowDate: string | number | Date,
  rowDesc: string,
  rowAmount: string,
  txnDate: string | number | Date,
  txnDesc: string,
  txnAmount: string,
) {
  const wasm = await loadExpentWasm();
  return wasm.calculate_match_score(
    BigInt(new Date(rowDate).getTime()),
    rowDesc,
    rowAmount,
    BigInt(new Date(txnDate).getTime()),
    txnDesc,
    txnAmount,
  );
}

/**
 * Performs a batch matching of statement rows against transactions using Rust/WASM.
 */
export async function matchStatementBatchWasm(statementRows: any[], transactions: any[]) {
  const wasm = await loadExpentWasm();
  return wasm.match_statement_batch(statementRows, transactions) as BatchMatchResult[];
}

/**
 * Performs a batch fuzzy search using Rust/WASM.
 */
export async function batchFuzzySearchWasm(query: string, items: string[], threshold: number = 0.5) {
  const wasm = await loadExpentWasm();
  return wasm.batch_fuzzy_search(query, items, threshold) as FuzzySearchResult[];
}

/**
 * Performs an advanced batch fuzzy search with multiple weighted fields using Rust/WASM.
 */
export async function advancedFuzzySearchWasm(query: string, items: SearchableItem[], threshold: number = 0.5) {
  const wasm = await loadExpentWasm();
  return wasm.advanced_fuzzy_search(query, items, threshold) as FuzzySearchResult[];
}

/**
 * Parses a numeric-like string into a number using Rust/WASM.
 */
export async function parseNumericLikeWasm(input: string) {
  const wasm = await loadExpentWasm();
  return wasm.parse_numeric_like(input) as number | undefined;
}

/**
 * Aggregates transaction metrics locally using Rust/WASM.
 */
export async function aggregateTransactionsWasm(transactions: Txn[]) {
  const wasm = await loadExpentWasm();
  return wasm.aggregate_transactions(transactions) as AggregatedMetrics;
}

/**
 * Generates a full dashboard summary locally using Rust/WASM.
 */
export async function generateDashboardSummaryWasm(transactions: any[], wallets: any[], categories: any[]) {
  const wasm = await loadExpentWasm();
  return wasm.generate_dashboard_summary(transactions, wallets, categories) as DashboardSummary;
}

/**
 * Detects subscription patterns locally using Rust/WASM.
 */
export async function detectSubscriptionPatternsWasm(transactions: TxnPattern[]) {
  const wasm = await loadExpentWasm();
  return wasm.detect_subscription_patterns(transactions) as DetectedSubscription[];
}

/**
 * Parses a CSV file into JSON locally using Rust/WASM.
 */
export async function parseCsvToWasm(data: Uint8Array) {
  const wasm = await loadExpentWasm();
  return wasm.parse_csv_to_json(data);
}

/**
 * Parses an Excel file into JSON locally using Rust/WASM.
 */
export async function parseExcelToWasm(data: Uint8Array) {
  const wasm = await loadExpentWasm();
  return wasm.parse_excel_to_json(data);
}

/**
 * Validates a transaction using Rust/WASM.
 */
export async function validateTransactionWasm(amount: string, purpose: string) {
  const wasm = await loadExpentWasm();
  return wasm.validate_transaction_wasm(amount, purpose);
}

/**
 * Validates a budget using Rust/WASM.
 */
export async function validateBudgetWasm(amount: string) {
  const wasm = await loadExpentWasm();
  return wasm.validate_budget_wasm(amount);
}

/**
 * Validates a wallet using Rust/WASM.
 */
export async function validateWalletWasm(name: string, balance: string) {
  const wasm = await loadExpentWasm();
  return wasm.validate_wallet_wasm(name, balance);
}

/**
 * Validates a contact using Rust/WASM.
 */
export async function validateContactWasm(name: string) {
  const wasm = await loadExpentWasm();
  return wasm.validate_contact_wasm(name);
}

/**
 * Validates a UPI ID (`handle@provider`) using Rust/WASM.
 *
 * Returns a `ValidationResult`; cast at the call site as we do for the other
 * validators. Tighter than RFC 5321 — handles are constrained to
 * `[a-zA-Z0-9._-]` (3-256 chars) and providers to `[a-zA-Z0-9.]` (2-64
 * chars), matching the NPCI namespace.
 */
export async function validateUpiIdWasm(upiId: string) {
  const wasm = await loadExpentWasm();
  return wasm.validate_upi_id_wasm(upiId);
}

/**
 * Validates an email address using Rust/WASM.
 *
 * Minimal RFC-5321-shaped check: one `@`, local <= 64 chars, domain with a
 * dot and >= 2-char TLD, no whitespace. Deliberately not a full RFC
 * validator.
 */
export async function validateEmailWasm(email: string) {
  const wasm = await loadExpentWasm();
  return wasm.validate_email_wasm(email);
}

/**
 * Validates a phone number using Rust/WASM.
 *
 * Strips `' -()+'` separators, then requires 7-15 ASCII digits (E.164 range).
 */
export async function validatePhoneWasm(phone: string) {
  const wasm = await loadExpentWasm();
  return wasm.validate_phone_wasm(phone);
}

/**
 * Formats a Decimal-precision amount as a currency string using Rust/WASM.
 *
 * Prefer JS `Intl.NumberFormat` for high-frequency render paths (it's
 * synchronous and locale-aware). Use this wrapper when you already have
 * wasm loaded — e.g. inside the OCR pipeline or other wasm-resident
 * processing — where the rust_decimal precision matters and avoiding
 * float drift is worth the async cost.
 *
 * INR uses Indian lakhs/crores grouping; other supported codes use Western
 * thousands. Unknown codes fall back to `<CODE> 1,234.56`. Returns
 * `undefined` for unparseable amounts.
 */
export async function formatCurrencyWasm(amount: string, currencyCode: string): Promise<string | undefined> {
  const wasm = await loadExpentWasm();
  return wasm.format_currency(amount, currencyCode);
}

/**
 * Best-effort guess at the currency referenced by a piece of free-form text
 * (OCR receipts, pasted text, …). Priority: ISO codes > unicode symbols >
 * `Rs`/`INR` shorthand. Returns the ISO code or `undefined`.
 */
export async function detectCurrencyFromTextWasm(text: string): Promise<string | undefined> {
  const wasm = await loadExpentWasm();
  return wasm.detect_currency_from_text(text);
}
