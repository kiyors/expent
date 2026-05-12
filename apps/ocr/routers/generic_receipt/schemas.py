from pydantic import BaseModel, Field
from typing import Optional, List


class LineItem(BaseModel):
    name: str = Field(description="Name of the item")
    quantity: int = Field(default=1)
    price: float = Field(description="Price of the item")


class OCRResponse(BaseModel):
    # Required by Rust OcrResult
    raw_text: str = Field(description="The full raw text extracted from the document")
    vendor: Optional[str] = Field(None, description="The vendor or shop name")
    amount: Optional[float] = Field(None, description="The total amount of the transaction")
    date: Optional[str] = Field(None, description="The date of the transaction in ISO or readable format")
    upi_id: Optional[str] = Field(None, description="The UPI ID if present")
    category_id: Optional[str] = Field(None, description="Assigned category ID")
    wallet_id: Optional[str] = Field(None, description="Assigned wallet ID")
    contact_id: Optional[str] = Field(None, description="Assigned contact ID")
    items: List[LineItem] = Field(default_factory=list)

    # Extra metadata for classification
    document_type: str = Field(description="payment_receipt | invoice | bank_statement | other")
    confidence_score: float = Field(default=1.0, description="Confidence score from 0.0 to 1.0")
