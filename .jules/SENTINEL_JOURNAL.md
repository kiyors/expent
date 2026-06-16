## 2026-05-11 - [Incomplete IDOR Validation in OCR API]
**Vulnerability:** IDOR (Insecure Direct Object Reference)
**Learning:** The `process_image_ocr_handler` implemented a security check for the primary S3 `key` but missed the same check for the optional `raw_key`. Since the background worker uses `raw_key` for high-resolution retries, an attacker could provide another user's `raw_key` and eventually receive the OCR results for it.
**Prevention:** When a payload contains multiple references to sensitive resources (like S3 keys), ensure that ALL references are validated for ownership against the authenticated session user.
- Found and fixed a Critical IDOR vulnerability in `crates/groups/src/p2p/`.
- **Pattern Identified:** Endpoints interacting with P2P requests and Ledger Tabs (e.g., `accept_p2p_request`, `reject_p2p_request`, `register_repayment`) implicitly trusted the `request_id` or `tab_id` provided by the authenticated user without verifying if the user was a party to the transaction.
- **Fix Applied:** Enforced explicit ownership/authorization checks before performing state-mutating actions:
  - `register_repayment`: Actor must be `creator_id` or `counterparty_id`.
  - `accept_p2p_request`: Actor's email must match `receiver_email`.
  - `reject_p2p_request`: Actor must be `sender_user_id` or have an email matching `receiver_email`.
- **Learning:** Always correlate the authenticated user session with the specific entity being acted upon in multi-party workflows (P2P, ledgers) to prevent unauthorized mutations.
- Found and fixed a High IDOR vulnerability in `crates/transactions/src/ops.rs` within `split_transaction`.
- **Pattern Identified:** When a user splits an existing transaction across recipients via `split_transaction`, the code retrieved the transaction by `txn_id` but failed to verify that the `txn.user_id` matches the `sender_id`.
- **Impact:** An attacker could split another user's transaction, effectively leaking the transaction metadata (amount, date, and purpose) to the recipients (or the attacker's own email).
- **Fix Applied:** Added an authorization check (`if txn.user_id != sender_id`) to ensure the transaction belongs to the sender before creating the P2P requests.
- **Learning:** Even inside core domain operations, fetching an entity by its primary ID must always be followed by an authorization check against the acting user's ID to prevent IDOR and data leakage.
