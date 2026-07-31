# Bolt Performance Journal

## ⚡ Reconciliation Upload Batching
Identified an N+1 query bottleneck in `apps/api/src/routes/reconciliation.rs` where uploaded statement rows were iterated sequentially and inserted into the database individually. By implementing a bulk processing routine in `crates/reconciliation/src/statement.rs`, we retrieve duplicate checking rows within the batch bounds using a single read and write the new records using `insert_many`.

This drops query load from 2n queries (duplicate check + insert) to exactly 2 queries per batch.

## `Intl` Formatter Caching in React

**Optimization:** Extracted `Intl.NumberFormat`, `Intl.DateTimeFormat`, and `Intl.RelativeTimeFormat` instantiations out of render loops into a module-level global cache utilizing `Map`.

**Why it matters:** The V8 engine has significant overhead when allocating and initializing complex locale-aware object instances like `Intl` formatters. When placed inline in heavily-rendered components like list items or data-table cell renderers, this overhead causes significant Main Thread blocking and reduces time-to-interactive.

**Result:** Caching these formatters by configuration fingerprint eliminates the redundant constructor calls, leading to substantially faster re-renders for large tables (e.g. 50+ rows).

**Caveat:** Caching mechanisms based on `JSON.stringify` on configuration objects are only viable when the shape and number of configuration variations are inherently bounded and predictable; unbounded parameters could cause a memory leak.

## Optimization: Intl Formatters and `serde_json` Cloning

- **Architecture-specific bottlenecks:** React data tables mapping over hundreds of records were repeatedly instantiating `Intl.NumberFormat`, `Intl.DateTimeFormat`, and `Intl.Collator` in the render loop. This caused main-thread blocking and excessive GC pressure.
- **Backend bottlenecks:** In Rust `serde_json::from_value(value.clone())` inside hot loops recursively allocates heap memory. Using `.take()` transfers ownership of the JSON value tree and drastically cuts down peak memory allocation during batch operations in OCR and P2P handlers.
- **Surprising edge cases:** Cache keys for `Intl` formatters need to explicitly handle `undefined` locales (e.g. `locale ?? "default"`) so the cache key doesn't evaluate to stringified `'undefined'`, which could collide with unexpected inputs or not be strictly deterministic in some JS runtimes.
