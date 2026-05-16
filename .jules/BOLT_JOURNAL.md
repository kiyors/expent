# Bolt Performance Journal

## ⚡ Reconciliation Upload Batching
Identified an N+1 query bottleneck in `apps/api/src/routes/reconciliation.rs` where uploaded statement rows were iterated sequentially and inserted into the database individually. By implementing a bulk processing routine in `crates/reconciliation/src/statement.rs`, we retrieve duplicate checking rows within the batch bounds using a single read and write the new records using `insert_many`.

This drops query load from 2n queries (duplicate check + insert) to exactly 2 queries per batch.
