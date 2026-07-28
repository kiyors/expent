/* @ts-self-types="./wasm.d.ts" */
import * as wasm from "./wasm_bg.wasm";
import { __wbg_set_wasm } from "./wasm_bg.js";

__wbg_set_wasm(wasm);

export {
    PeriodBounds, SavingsProjection, SpendingVelocity, advanced_fuzzy_search, aggregate_transactions, batch_fuzzy_search, calculate_budget_percentage, calculate_match_score, calculate_spending_velocity, detect_currency_from_text, detect_subscription_patterns, format_currency, fuzzy_score, generate_dashboard_summary, get_period_bounds, is_transaction_in_period, match_statement_batch, normalize_text, parse_csv_to_json, parse_excel_to_json, parse_numeric_like, phonetic_encode, project_savings_goal, validate_budget_wasm, validate_contact_wasm, validate_email_wasm, validate_phone_wasm, validate_transaction_wasm, validate_upi_id_wasm, validate_wallet_wasm
} from "./wasm_bg.js";
