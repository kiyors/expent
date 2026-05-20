import type { BudgetPeriod } from "@expent/types";

/**
 * Loads the expent_wasm module.
 * Note: This must be called inside a useEffect or after a user interaction
 * because it is an async WebAssembly module.
 */
export async function loadExpentWasm() {
  const wasm = await import("@expent/wasm");
  // Initialize the WASM module (required for --target web)
  await wasm.default();
  return wasm;
}

/**
 * Calculates budget percentage consumption using Rust/WASM.
 */
export async function calculateBudgetPercentageWasm(spent: string, limit: string) {
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
  return wasm.calculate_spending_velocity(spent, limit, period);
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
  return wasm.project_savings_goal(currentBalance, targetAmount, monthlyIncome, monthlyExpenses);
}
