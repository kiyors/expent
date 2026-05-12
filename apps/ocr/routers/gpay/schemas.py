from pydantic import BaseModel, Field
from typing import Optional, Literal


class GPayExtraction(BaseModel):
    # Core Ledger Data
    amount: float
    direction: Literal["IN", "OUT"]  # "OUT" if "To", "IN" if "From"
    datetime_str: Optional[str] = Field(None, description="Exact date and time string e.g. '11 Mar 2026, 1:51 pm'")
    status: Literal["COMPLETED", "PENDING", "FAILED"]

    # Counterparty Info (The person/business you are interacting with)
    counterparty_name: str = Field(description="Name of the person or business")
    counterparty_phone: Optional[str] = None
    counterparty_upi_id: Optional[str] = None
    is_merchant: bool = Field(
        description="True if the counterparty is a business/shop (e.g., has 'vyapar' in UPI or sounds like a store)"
    )

    # Transaction Metadata (Often missing in 'Immediate' screens)
    upi_transaction_id: Optional[str] = None
    google_transaction_id: Optional[str] = None
    source_bank_account: Optional[str] = Field(description="E.g., 'ICICI Bank 0972'")
    category_id: Optional[str] = Field(None, description="Assigned category ID")
    wallet_id: Optional[str] = Field(None, description="Assigned wallet ID")
    contact_id: Optional[str] = Field(None, description="Assigned contact ID")
    confidence_score: float = Field(default=1.0, description="Confidence score from 0.0 to 1.0")
