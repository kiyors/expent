/* @ts-self-types="./wasm.d.ts" */

export class PeriodBounds {
    static __wrap(ptr) {
        const obj = Object.create(PeriodBounds.prototype);
        obj.__wbg_ptr = ptr;
        PeriodBoundsFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        PeriodBoundsFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_periodbounds_free(ptr, 0);
    }
    /**
     * @returns {bigint}
     */
    get end_ms() {
        const ret = wasm.__wbg_get_periodbounds_end_ms(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {bigint}
     */
    get start_ms() {
        const ret = wasm.__wbg_get_periodbounds_start_ms(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {bigint} arg0
     */
    set end_ms(arg0) {
        wasm.__wbg_set_periodbounds_end_ms(this.__wbg_ptr, arg0);
    }
    /**
     * @param {bigint} arg0
     */
    set start_ms(arg0) {
        wasm.__wbg_set_periodbounds_start_ms(this.__wbg_ptr, arg0);
    }
}
if (Symbol.dispose) PeriodBounds.prototype[Symbol.dispose] = PeriodBounds.prototype.free;

export class SavingsProjection {
    static __wrap(ptr) {
        const obj = Object.create(SavingsProjection.prototype);
        obj.__wbg_ptr = ptr;
        SavingsProjectionFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        SavingsProjectionFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_savingsprojection_free(ptr, 0);
    }
    /**
     * @returns {boolean}
     */
    get is_attainable() {
        const ret = wasm.__wbg_get_savingsprojection_is_attainable(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {number}
     */
    get monthly_contribution() {
        const ret = wasm.__wbg_get_savingsprojection_monthly_contribution(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get months_to_goal() {
        const ret = wasm.__wbg_get_savingsprojection_months_to_goal(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {boolean} arg0
     */
    set is_attainable(arg0) {
        wasm.__wbg_set_savingsprojection_is_attainable(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set monthly_contribution(arg0) {
        wasm.__wbg_set_savingsprojection_monthly_contribution(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set months_to_goal(arg0) {
        wasm.__wbg_set_savingsprojection_months_to_goal(this.__wbg_ptr, arg0);
    }
}
if (Symbol.dispose) SavingsProjection.prototype[Symbol.dispose] = SavingsProjection.prototype.free;

export class SpendingVelocity {
    static __wrap(ptr) {
        const obj = Object.create(SpendingVelocity.prototype);
        obj.__wbg_ptr = ptr;
        SpendingVelocityFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        SpendingVelocityFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_spendingvelocity_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get daily_burn_rate() {
        const ret = wasm.__wbg_get_spendingvelocity_daily_burn_rate(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {boolean}
     */
    get is_overpacing() {
        const ret = wasm.__wbg_get_spendingvelocity_is_overpacing(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {number}
     */
    get projected_total() {
        const ret = wasm.__wbg_get_spendingvelocity_projected_total(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get target_daily_rate() {
        const ret = wasm.__wbg_get_spendingvelocity_target_daily_rate(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set daily_burn_rate(arg0) {
        wasm.__wbg_set_spendingvelocity_daily_burn_rate(this.__wbg_ptr, arg0);
    }
    /**
     * @param {boolean} arg0
     */
    set is_overpacing(arg0) {
        wasm.__wbg_set_spendingvelocity_is_overpacing(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set projected_total(arg0) {
        wasm.__wbg_set_spendingvelocity_projected_total(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set target_daily_rate(arg0) {
        wasm.__wbg_set_spendingvelocity_target_daily_rate(this.__wbg_ptr, arg0);
    }
}
if (Symbol.dispose) SpendingVelocity.prototype[Symbol.dispose] = SpendingVelocity.prototype.free;

/**
 * @param {string} query
 * @param {any} items
 * @param {number} threshold
 * @returns {any}
 */
export function advanced_fuzzy_search(query, items, threshold) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(query, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.advanced_fuzzy_search(retptr, ptr0, len0, addHeapObject(items), threshold);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return takeObject(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {any} transactions
 * @returns {any}
 */
export function aggregate_transactions(transactions) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.aggregate_transactions(retptr, addHeapObject(transactions));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return takeObject(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {string} query
 * @param {string[]} items
 * @param {number} threshold
 * @returns {any}
 */
export function batch_fuzzy_search(query, items, threshold) {
    const ptr0 = passStringToWasm0(query, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passArrayJsValueToWasm0(items, wasm.__wbindgen_export);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.batch_fuzzy_search(ptr0, len0, ptr1, len1, threshold);
    return takeObject(ret);
}

/**
 * Compute `spent / limit * 100` as a Decimal-precise percentage string.
 *
 * Returns `None` (becomes `undefined` in JS) when either input fails to parse,
 * surfacing bad caller data instead of silently reporting "0". A zero `limit`
 * is still treated as 0% so dividing-by-zero doesn't blow up.
 * @param {string} spent
 * @param {string} limit
 * @returns {string | undefined}
 */
export function calculate_budget_percentage(spent, limit) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(spent, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(limit, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len1 = WASM_VECTOR_LEN;
        wasm.calculate_budget_percentage(retptr, ptr0, len0, ptr1, len1);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        let v3;
        if (r0 !== 0) {
            v3 = getStringFromWasm0(r0, r1).slice();
            wasm.__wbindgen_export4(r0, r1 * 1, 1);
        }
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {bigint} row_date_ms
 * @param {string} row_desc
 * @param {string} row_amount
 * @param {bigint} txn_date_ms
 * @param {string} txn_desc
 * @param {string} txn_amount
 * @returns {number}
 */
export function calculate_match_score(row_date_ms, row_desc, row_amount, txn_date_ms, txn_desc, txn_amount) {
    const ptr0 = passStringToWasm0(row_desc, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(row_amount, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passStringToWasm0(txn_desc, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len2 = WASM_VECTOR_LEN;
    const ptr3 = passStringToWasm0(txn_amount, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len3 = WASM_VECTOR_LEN;
    const ret = wasm.calculate_match_score(row_date_ms, ptr0, len0, ptr1, len1, txn_date_ms, ptr2, len2, ptr3, len3);
    return ret;
}

/**
 * @param {string} spent
 * @param {string} limit
 * @param {string} period
 * @returns {SpendingVelocity | undefined}
 */
export function calculate_spending_velocity(spent, limit, period) {
    const ptr0 = passStringToWasm0(spent, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(limit, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passStringToWasm0(period, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len2 = WASM_VECTOR_LEN;
    const ret = wasm.calculate_spending_velocity(ptr0, len0, ptr1, len1, ptr2, len2);
    return ret === 0 ? undefined : SpendingVelocity.__wrap(ret);
}

/**
 * Best-effort guess at the currency a piece of free-form text refers to.
 *
 * Priority: explicit ISO codes (case-insensitive whole-word match) win over
 * symbols, because OCR pipelines often see "INR 250" alongside a unicode
 * rupee sign, and the code is the surer signal. Returns `None` when nothing
 * matches — callers should keep their existing fallback.
 * @param {string} text
 * @returns {string | undefined}
 */
export function detect_currency_from_text(text) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(text, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.detect_currency_from_text(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        let v2;
        if (r0 !== 0) {
            v2 = getStringFromWasm0(r0, r1).slice();
            wasm.__wbindgen_export4(r0, r1 * 1, 1);
        }
        return v2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {any} transactions
 * @returns {any}
 */
export function detect_subscription_patterns(transactions) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.detect_subscription_patterns(retptr, addHeapObject(transactions));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return takeObject(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * Format a Decimal-precision amount as a currency string.
 *
 * Returns `None` when `amount` does not parse. INR uses Indian
 * lakhs/crores grouping (e.g. `₹12,34,567.89`); other supported codes use
 * Western grouping (`$1,234,567.89`). Unknown codes are still formatted with
 * the code itself as the prefix (e.g. `XYZ 1,234.56`) — the function never
 * silently substitutes a different currency.
 * @param {string} amount
 * @param {string} currency_code
 * @returns {string | undefined}
 */
export function format_currency(amount, currency_code) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(amount, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(currency_code, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len1 = WASM_VECTOR_LEN;
        wasm.format_currency(retptr, ptr0, len0, ptr1, len1);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        let v3;
        if (r0 !== 0) {
            v3 = getStringFromWasm0(r0, r1).slice();
            wasm.__wbindgen_export4(r0, r1 * 1, 1);
        }
        return v3;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {string} a
 * @param {string} b
 * @returns {number}
 */
export function fuzzy_score(a, b) {
    const ptr0 = passStringToWasm0(a, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(b, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.fuzzy_score(ptr0, len0, ptr1, len1);
    return ret;
}

/**
 * @param {any} transactions
 * @param {any} wallets
 * @param {any} categories
 * @returns {any}
 */
export function generate_dashboard_summary(transactions, wallets, categories) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.generate_dashboard_summary(retptr, addHeapObject(transactions), addHeapObject(wallets), addHeapObject(categories));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return takeObject(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {string} period
 * @returns {PeriodBounds | undefined}
 */
export function get_period_bounds(period) {
    const ptr0 = passStringToWasm0(period, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.get_period_bounds(ptr0, len0);
    return ret === 0 ? undefined : PeriodBounds.__wrap(ret);
}

/**
 * @param {bigint} txn_date_ms
 * @param {string} period
 * @returns {boolean}
 */
export function is_transaction_in_period(txn_date_ms, period) {
    const ptr0 = passStringToWasm0(period, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.is_transaction_in_period(txn_date_ms, ptr0, len0);
    return ret !== 0;
}

/**
 * @param {any} statement_rows
 * @param {any} transactions
 * @returns {any}
 */
export function match_statement_batch(statement_rows, transactions) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.match_statement_batch(retptr, addHeapObject(statement_rows), addHeapObject(transactions));
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return takeObject(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {string} text
 * @returns {string}
 */
export function normalize_text(text) {
    let deferred2_0;
    let deferred2_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(text, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.normalize_text(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred2_0 = r0;
        deferred2_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export4(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {Uint8Array} data
 * @returns {any}
 */
export function parse_csv_to_json(data) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        wasm.parse_csv_to_json(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return takeObject(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {Uint8Array} data
 * @returns {any}
 */
export function parse_excel_to_json(data) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        wasm.parse_excel_to_json(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return takeObject(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {string} input
 * @returns {number | undefined}
 */
export function parse_numeric_like(input) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(input, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.parse_numeric_like(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r2 = getDataViewMemory0().getFloat64(retptr + 8 * 1, true);
        return r0 === 0 ? undefined : r2;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * @param {string} text
 * @returns {string}
 */
export function phonetic_encode(text) {
    let deferred2_0;
    let deferred2_1;
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passStringToWasm0(text, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.phonetic_encode(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        deferred2_0 = r0;
        deferred2_1 = r1;
        return getStringFromWasm0(r0, r1);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        wasm.__wbindgen_export4(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {string} current_balance
 * @param {string} target_amount
 * @param {string} monthly_income
 * @param {string} monthly_expenses
 * @returns {SavingsProjection | undefined}
 */
export function project_savings_goal(current_balance, target_amount, monthly_income, monthly_expenses) {
    const ptr0 = passStringToWasm0(current_balance, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(target_amount, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passStringToWasm0(monthly_income, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len2 = WASM_VECTOR_LEN;
    const ptr3 = passStringToWasm0(monthly_expenses, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len3 = WASM_VECTOR_LEN;
    const ret = wasm.project_savings_goal(ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3);
    return ret === 0 ? undefined : SavingsProjection.__wrap(ret);
}

/**
 * @param {string} amount
 * @returns {any}
 */
export function validate_budget_wasm(amount) {
    const ptr0 = passStringToWasm0(amount, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.validate_budget_wasm(ptr0, len0);
    return takeObject(ret);
}

/**
 * @param {string} name
 * @returns {any}
 */
export function validate_contact_wasm(name) {
    const ptr0 = passStringToWasm0(name, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.validate_contact_wasm(ptr0, len0);
    return takeObject(ret);
}

/**
 * @param {string} email
 * @returns {any}
 */
export function validate_email_wasm(email) {
    const ptr0 = passStringToWasm0(email, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.validate_email_wasm(ptr0, len0);
    return takeObject(ret);
}

/**
 * @param {string} phone
 * @returns {any}
 */
export function validate_phone_wasm(phone) {
    const ptr0 = passStringToWasm0(phone, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.validate_phone_wasm(ptr0, len0);
    return takeObject(ret);
}

/**
 * @param {string} amount
 * @param {string} purpose
 * @returns {any}
 */
export function validate_transaction_wasm(amount, purpose) {
    const ptr0 = passStringToWasm0(amount, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(purpose, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.validate_transaction_wasm(ptr0, len0, ptr1, len1);
    return takeObject(ret);
}

/**
 * @param {string} upi_id
 * @returns {any}
 */
export function validate_upi_id_wasm(upi_id) {
    const ptr0 = passStringToWasm0(upi_id, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.validate_upi_id_wasm(ptr0, len0);
    return takeObject(ret);
}

/**
 * @param {string} name
 * @param {string} balance
 * @returns {any}
 */
export function validate_wallet_wasm(name, balance) {
    const ptr0 = passStringToWasm0(name, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(balance, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.validate_wallet_wasm(ptr0, len0, ptr1, len1);
    return takeObject(ret);
}
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg_Error_bce6d499ff0a4aff: function(arg0, arg1) {
            const ret = Error(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        },
        __wbg_String_8564e559799eccda: function(arg0, arg1) {
            const ret = String(getObject(arg1));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_boolean_get_2304fb8c853028c8: function(arg0) {
            const v = getObject(arg0);
            const ret = typeof(v) === 'boolean' ? v : undefined;
            return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
        },
        __wbg___wbindgen_debug_string_edece8177ad01481: function(arg0, arg1) {
            const ret = debugString(getObject(arg1));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_in_07056af4f902c445: function(arg0, arg1) {
            const ret = getObject(arg0) in getObject(arg1);
            return ret;
        },
        __wbg___wbindgen_is_function_5cd60d5cf78b4eef: function(arg0) {
            const ret = typeof(getObject(arg0)) === 'function';
            return ret;
        },
        __wbg___wbindgen_is_object_b4593df85baada48: function(arg0) {
            const val = getObject(arg0);
            const ret = typeof(val) === 'object' && val !== null;
            return ret;
        },
        __wbg___wbindgen_is_undefined_35bb9f4c7fd651d5: function(arg0) {
            const ret = getObject(arg0) === undefined;
            return ret;
        },
        __wbg___wbindgen_jsval_loose_eq_0ad77b7717db155c: function(arg0, arg1) {
            const ret = getObject(arg0) == getObject(arg1);
            return ret;
        },
        __wbg___wbindgen_number_get_f73a1244370fcc2c: function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'number' ? obj : undefined;
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        },
        __wbg___wbindgen_string_get_d109740c0d18f4d7: function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'string' ? obj : undefined;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_throw_9c31b086c2b26051: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_call_13665d9f14390edc: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).call(getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_done_54b8da57023b7ed2: function(arg0) {
            const ret = getObject(arg0).done;
            return ret;
        },
        __wbg_getTime_09f1dd40a44edb30: function(arg0) {
            const ret = getObject(arg0).getTime();
            return ret;
        },
        __wbg_get_3e9a707ab7d352eb: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(getObject(arg0), getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_get_unchecked_1dfe6d05ad91d9b7: function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return addHeapObject(ret);
        },
        __wbg_get_with_ref_key_6412cf3094599694: function(arg0, arg1) {
            const ret = getObject(arg0)[getObject(arg1)];
            return addHeapObject(ret);
        },
        __wbg_instanceof_ArrayBuffer_53db37b06f6b9afe: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof ArrayBuffer;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Uint8Array_abd07d4bd221d50b: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Uint8Array;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_isArray_94898ed3aad6947b: function(arg0) {
            const ret = Array.isArray(getObject(arg0));
            return ret;
        },
        __wbg_iterator_1441b47f341dc34f: function() {
            const ret = Symbol.iterator;
            return addHeapObject(ret);
        },
        __wbg_length_2591a0f4f659a55c: function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        },
        __wbg_length_56fcd3e2b7e0299d: function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        },
        __wbg_new_02d162bc6cf02f60: function() {
            const ret = new Object();
            return addHeapObject(ret);
        },
        __wbg_new_070df68d66325372: function() {
            const ret = new Map();
            return addHeapObject(ret);
        },
        __wbg_new_0_2722fcdb71a888a6: function() {
            const ret = new Date();
            return addHeapObject(ret);
        },
        __wbg_new_310879b66b6e95e1: function() {
            const ret = new Array();
            return addHeapObject(ret);
        },
        __wbg_new_7ddec6de44ff8f5d: function(arg0) {
            const ret = new Uint8Array(getObject(arg0));
            return addHeapObject(ret);
        },
        __wbg_next_2a4e19f4f5083b0f: function(arg0) {
            const ret = getObject(arg0).next;
            return addHeapObject(ret);
        },
        __wbg_next_6429a146bf756f93: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).next();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_prototypesetcall_5f9bdc8d75e07276: function(arg0, arg1, arg2) {
            Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), getObject(arg2));
        },
        __wbg_set_6be42768c690e380: function(arg0, arg1, arg2) {
            getObject(arg0)[takeObject(arg1)] = takeObject(arg2);
        },
        __wbg_set_78ea6a19f4818587: function(arg0, arg1, arg2) {
            getObject(arg0)[arg1 >>> 0] = takeObject(arg2);
        },
        __wbg_set_facb7a5914e0fa39: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).set(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        },
        __wbg_value_9cc0518af87a489c: function(arg0) {
            const ret = getObject(arg0).value;
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000001: function(arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000003: function(arg0) {
            // Cast intrinsic for `U64 -> Externref`.
            const ret = BigInt.asUintN(64, arg0);
            return addHeapObject(ret);
        },
        __wbindgen_object_clone_ref: function(arg0) {
            const ret = getObject(arg0);
            return addHeapObject(ret);
        },
        __wbindgen_object_drop_ref: function(arg0) {
            takeObject(arg0);
        },
    };
    return {
        __proto__: null,
        "./wasm_bg.js": import0,
    };
}

const PeriodBoundsFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_periodbounds_free(ptr, 1));
const SavingsProjectionFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_savingsprojection_free(ptr, 1));
const SpendingVelocityFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_spendingvelocity_free(ptr, 1));

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function dropObject(idx) {
    if (idx < 1028) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function getObject(idx) { return heap[idx]; }

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_export3(addHeapObject(e));
    }
}

let heap = new Array(1024).fill(undefined);
heap.push(undefined, null, true, false);

let heap_next = heap.length;

function isLikeNone(x) {
    return x === undefined || x === null;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passArrayJsValueToWasm0(array, malloc) {
    const ptr = malloc(array.length * 4, 4) >>> 0;
    const mem = getDataViewMemory0();
    for (let i = 0; i < array.length; i++) {
        mem.setUint32(ptr + 4 * i, addHeapObject(array[i]), true);
    }
    WASM_VECTOR_LEN = array.length;
    return ptr;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
