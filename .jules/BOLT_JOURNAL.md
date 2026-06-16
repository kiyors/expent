# Bolt Performance Journal

## ⚡ Reconciliation Upload Batching
Identified an N+1 query bottleneck in `apps/api/src/routes/reconciliation.rs` where uploaded statement rows were iterated sequentially and inserted into the database individually. By implementing a bulk processing routine in `crates/reconciliation/src/statement.rs`, we retrieve duplicate checking rows within the batch bounds using a single read and write the new records using `insert_many`.

This drops query load from 2n queries (duplicate check + insert) to exactly 2 queries per batch.

## `Intl` Formatter Caching in React

**Optimization:** Extracted `Intl.NumberFormat`, `Intl.DateTimeFormat`, and `Intl.RelativeTimeFormat` instantiations out of render loops into a module-level global cache utilizing `Map`.

**Why it matters:** The V8 engine has significant overhead when allocating and initializing complex locale-aware object instances like `Intl` formatters. When placed inline in heavily-rendered components like list items or data-table cell renderers, this overhead causes significant Main Thread blocking and reduces time-to-interactive.

**Result:** Caching these formatters by configuration fingerprint eliminates the redundant constructor calls, leading to substantially faster re-renders for large tables (e.g. 50+ rows).

**Caveat:** Caching mechanisms based on `JSON.stringify` on configuration objects are only viable when the shape and number of configuration variations are inherently bounded and predictable; unbounded parameters could cause a memory leak.

## `get_monthly_trends` and `get_weekly_trends` Hot Loop Allocation

**Optimization:** Replaced database string formatting (`to_char`/`strftime`) and Rust-level `String` keys with `(i32, u32)` tuples and `chrono::NaiveDate` for the internal `BTreeMap` used to calculate transaction trends.

**Why it matters:** The original code used `date.format(...)` inside initialization loops and `.split('-')` on string keys fetched from the database on every transaction record to update trends. By fetching integer parts (Year/Month/Day) directly from the database and using tuples/`NaiveDate` structs as map keys, we eliminate all string allocation and parsing overhead in these aggregation loops.

**Result:** O(N) heap allocations for formatting and string splitting were removed, yielding a tighter, zero-allocation data aggregation loop and faster dashboard summary queries.

**Caveat:** The `EXTRACT()` function in PostgreSQL 14+ returns a `numeric` type (not `double precision`). When mapping to `i32` in Rust's `sea-orm` `FromQueryResult`, you must wrap the extraction in `CAST(... AS INTEGER)`.
