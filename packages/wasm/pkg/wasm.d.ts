/* tslint:disable */
/* eslint-disable */

export class PeriodBounds {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    end_ms: bigint;
    start_ms: bigint;
}

export class SavingsProjection {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    is_attainable: boolean;
    monthly_contribution: number;
    months_to_goal: number;
}

export class SpendingVelocity {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    daily_burn_rate: number;
    is_overpacing: boolean;
    projected_total: number;
    target_daily_rate: number;
}

export function advanced_fuzzy_search(query: string, items: any, threshold: number): any;

export function aggregate_transactions(transactions: any): any;

export function batch_fuzzy_search(query: string, items: string[], threshold: number): any;

/**
 * Compute `spent / limit * 100` as a Decimal-precise percentage string.
 *
 * Returns `None` (becomes `undefined` in JS) when either input fails to parse,
 * surfacing bad caller data instead of silently reporting "0". A zero `limit`
 * is still treated as 0% so dividing-by-zero doesn't blow up.
 */
export function calculate_budget_percentage(spent: string, limit: string): string | undefined;

export function calculate_match_score(row_date_ms: bigint, row_desc: string, row_amount: string, txn_date_ms: bigint, txn_desc: string, txn_amount: string): number;

export function calculate_spending_velocity(spent: string, limit: string, period: string): SpendingVelocity | undefined;

/**
 * Best-effort guess at the currency a piece of free-form text refers to.
 *
 * Priority: explicit ISO codes (case-insensitive whole-word match) win over
 * symbols, because OCR pipelines often see "INR 250" alongside a unicode
 * rupee sign, and the code is the surer signal. Returns `None` when nothing
 * matches — callers should keep their existing fallback.
 */
export function detect_currency_from_text(text: string): string | undefined;

export function detect_subscription_patterns(transactions: any): any;

/**
 * Format a Decimal-precision amount as a currency string.
 *
 * Returns `None` when `amount` does not parse. INR uses Indian
 * lakhs/crores grouping (e.g. `₹12,34,567.89`); other supported codes use
 * Western grouping (`$1,234,567.89`). Unknown codes are still formatted with
 * the code itself as the prefix (e.g. `XYZ 1,234.56`) — the function never
 * silently substitutes a different currency.
 */
export function format_currency(amount: string, currency_code: string): string | undefined;

export function fuzzy_score(a: string, b: string): number;

export function generate_dashboard_summary(transactions: any, wallets: any, categories: any): any;

export function get_period_bounds(period: string): PeriodBounds | undefined;

export function is_transaction_in_period(txn_date_ms: bigint, period: string): boolean;

export function match_statement_batch(statement_rows: any, transactions: any): any;

export function normalize_text(text: string): string;

export function parse_csv_to_json(data: Uint8Array): any;

export function parse_excel_to_json(data: Uint8Array): any;

export function parse_numeric_like(input: string): number | undefined;

export function phonetic_encode(text: string): string;

export function project_savings_goal(current_balance: string, target_amount: string, monthly_income: string, monthly_expenses: string): SavingsProjection | undefined;

export function validate_budget_wasm(amount: string): any;

export function validate_contact_wasm(name: string): any;

export function validate_email_wasm(email: string): any;

export function validate_phone_wasm(phone: string): any;

export function validate_transaction_wasm(amount: string, purpose: string): any;

export function validate_upi_id_wasm(upi_id: string): any;

export function validate_wallet_wasm(name: string, balance: string): any;
