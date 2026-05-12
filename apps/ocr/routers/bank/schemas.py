from pydantic import BaseModel, Field
from typing import List, Optional


class BankTransaction(BaseModel):
    transaction_date: str = Field(description="The date of the transaction. MUST ALWAYS be in YYYY-MM-DD format.")
    description: str = Field(description="The full particulars or description of the transaction")
    mode: Optional[str] = Field(None, description="The mode of transaction, e.g., UPI, NEFT, INF, IMPS")
    debit_amount: Optional[float] = Field(None, description="Amount withdrawn/debited. Null if deposit.")
    credit_amount: Optional[float] = Field(None, description="Amount deposited/credited. Null if withdrawal.")
    balance: Optional[float] = Field(None, description="The account balance after the transaction")
    contact_name: Optional[str] = Field(
        None, description="Extracted name of the sender/receiver (e.g. from UPI or IMPS)"
    )
    upi_id: Optional[str] = Field(None, description="Extracted UPI ID, if available")
    reference_number: Optional[str] = Field(None, description="Bank reference number or transaction ID")
    category_id: Optional[str] = Field(None, description="Assigned category ID")
    wallet_id: Optional[str] = Field(None, description="Assigned wallet ID")
    raw_particulars: Optional[str] = Field(None, description="The original un-truncated particulars string")


class BankStatementResponse(BaseModel):
    transactions: List[BankTransaction]
    bank_name: str
    account_number: Optional[str]
    statement_period: str


class BankExtractionResult(BaseModel):
    """The final object returned to Rust after processing a bank statement."""

    raw_text: str = Field(description="The full raw text extracted from the document")
    doc_type: str = Field(default="bank_statement")
    confidence_score: float = Field(default=1.0)
    bank_data: BankStatementResponse
