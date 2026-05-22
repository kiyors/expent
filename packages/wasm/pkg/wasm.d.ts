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

export function calculate_budget_percentage(spent: string, limit: string): string;

export function calculate_match_score(row_date_ms: bigint, row_desc: string, row_amount: string, txn_date_ms: bigint, txn_desc: string, txn_amount: string): number;

export function calculate_spending_velocity(spent: string, limit: string, period: string): SpendingVelocity | undefined;

export function detect_subscription_patterns(transactions: any): any;

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

export function validate_transaction_wasm(amount: string, purpose: string): any;

export function validate_wallet_wasm(name: string, balance: string): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_get_periodbounds_end_ms: (a: number) => bigint;
    readonly __wbg_get_periodbounds_start_ms: (a: number) => bigint;
    readonly __wbg_get_savingsprojection_is_attainable: (a: number) => number;
    readonly __wbg_get_savingsprojection_monthly_contribution: (a: number) => number;
    readonly __wbg_get_savingsprojection_months_to_goal: (a: number) => number;
    readonly __wbg_get_spendingvelocity_is_overpacing: (a: number) => number;
    readonly __wbg_get_spendingvelocity_projected_total: (a: number) => number;
    readonly __wbg_get_spendingvelocity_target_daily_rate: (a: number) => number;
    readonly __wbg_periodbounds_free: (a: number, b: number) => void;
    readonly __wbg_set_periodbounds_end_ms: (a: number, b: bigint) => void;
    readonly __wbg_set_periodbounds_start_ms: (a: number, b: bigint) => void;
    readonly __wbg_set_savingsprojection_is_attainable: (a: number, b: number) => void;
    readonly __wbg_set_savingsprojection_monthly_contribution: (a: number, b: number) => void;
    readonly __wbg_set_savingsprojection_months_to_goal: (a: number, b: number) => void;
    readonly __wbg_set_spendingvelocity_is_overpacing: (a: number, b: number) => void;
    readonly __wbg_set_spendingvelocity_projected_total: (a: number, b: number) => void;
    readonly __wbg_set_spendingvelocity_target_daily_rate: (a: number, b: number) => void;
    readonly __wbg_spendingvelocity_free: (a: number, b: number) => void;
    readonly advanced_fuzzy_search: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly aggregate_transactions: (a: number, b: number) => void;
    readonly batch_fuzzy_search: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly calculate_budget_percentage: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly calculate_match_score: (a: bigint, b: number, c: number, d: number, e: number, f: bigint, g: number, h: number, i: number, j: number) => number;
    readonly calculate_spending_velocity: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly detect_subscription_patterns: (a: number, b: number) => void;
    readonly fuzzy_score: (a: number, b: number, c: number, d: number) => number;
    readonly generate_dashboard_summary: (a: number, b: number, c: number, d: number) => void;
    readonly get_period_bounds: (a: number, b: number) => number;
    readonly is_transaction_in_period: (a: bigint, b: number, c: number) => number;
    readonly match_statement_batch: (a: number, b: number, c: number) => void;
    readonly normalize_text: (a: number, b: number, c: number) => void;
    readonly parse_csv_to_json: (a: number, b: number, c: number) => void;
    readonly parse_excel_to_json: (a: number, b: number, c: number) => void;
    readonly parse_numeric_like: (a: number, b: number, c: number) => void;
    readonly phonetic_encode: (a: number, b: number, c: number) => void;
    readonly project_savings_goal: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => number;
    readonly validate_budget_wasm: (a: number, b: number) => number;
    readonly validate_contact_wasm: (a: number, b: number) => number;
    readonly validate_transaction_wasm: (a: number, b: number, c: number, d: number) => number;
    readonly validate_wallet_wasm: (a: number, b: number, c: number, d: number) => number;
    readonly __wbg_get_spendingvelocity_daily_burn_rate: (a: number) => number;
    readonly __wbg_set_spendingvelocity_daily_burn_rate: (a: number, b: number) => void;
    readonly __wbg_savingsprojection_free: (a: number, b: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
