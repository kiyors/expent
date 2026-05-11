import os
import json
import io
import asyncio
from typing import Union, Literal, Optional, Any, List
from pydantic import BaseModel, Field
from google import genai
from google.genai import types
import easyocr

# Import schemas
from routers.gpay.schemas import GPayExtraction
from routers.generic_receipt.schemas import OCRResponse as GenericOCRResponse
from routers.bank.schemas import BankStatementResponse, BankExtractionResult
from utils import get_media_type, extract_pdf_text, parse_csv, parse_excel


class UnifiedExtraction(BaseModel):
    doc_type: Literal["GPAY", "BANK_STATEMENT", "GENERIC"]
    confidence_score: float = Field(default=1.0)
    raw_text: Optional[str] = Field(None, description="The full raw text extracted from the document")

    # Specific data fields (only one will be populated based on doc_type)
    gpay_data: Optional[GPayExtraction] = Field(None, description="Populated if doc_type is GPAY")
    bank_data: Optional[BankStatementResponse] = Field(None, description="Populated if doc_type is BANK_STATEMENT")
    generic_data: Optional[GenericOCRResponse] = Field(None, description="Populated if doc_type is GENERIC")


class OCREngine:
    def __init__(self, api_key: str = None):
        key = api_key or os.getenv("GOOGLE_API_KEY")
        self.client = genai.Client(api_key=key)
        self.model_name = os.getenv("GEMINI_MODEL", "gemini-2.5-flash")
        self._reader = None

    @property
    def reader(self):
        """Lazy load EasyOCR reader."""
        if self._reader is None:
            self._reader = easyocr.Reader(["en"])
        return self._reader

    def get_system_prompt(self) -> str:
        return """
You are an advanced financial data extraction engine. Your task is to analyze the provided document (image, PDF, CSV, or Excel) and extract structured information.

STEP 1: CLASSIFY THE DOCUMENT
- GPAY: Google Pay payment confirmation screenshots (digital).
- BANK_STATEMENT: Monthly bank statements (PDF, CSV, or Image tables).
- GENERIC: Retail receipts, invoices, or other payment proofs.

STEP 2: EXTRACT DATA BASED ON TYPE

--- RULES FOR GPAY ---
- Extract 'amount', 'direction' (IN if 'From', OUT if 'To'), 'status' (COMPLETED/PENDING/FAILED), 'counterparty_name', and transaction IDs.
- 'is_merchant' is true if the name sounds like a business or UPI has 'vyapar'.

--- RULES FOR BANK_STATEMENT ---
- Identify the bank (e.g., ICICI, HDFC, SBI).
- Extract ALL transactions. 
- For ICICI: Stitch multi-line 'PARTICULARS' into a single description.
- Extract 'contact_name' and 'upi_id' from transaction descriptions if available.
- Map withdrawals to 'debit_amount' and deposits to 'credit_amount'.

--- RULES FOR GENERIC ---
- Extract 'vendor', total 'amount', 'date', and 'items' (if visible).
- Normalize date to YYYY-MM-DD if possible.

STEP 3: FORMAT OUTPUT
- Return ONLY a valid JSON object matching the requested schema.
- Use snake_case for all keys.
- If a field is missing, set it to null.
- Ensure 'confidence_score' reflects your certainty (0.0 to 1.0).
"""

    async def extract_from_bytes(self, data: bytes, filename: str) -> dict:
        media_type = get_media_type(filename)

        # 1. Augment with text extraction for non-image formats
        extracted_text = ""
        if media_type == "application/pdf":
            extracted_text = extract_pdf_text(data)
        elif media_type == "text/csv":
            extracted_text = parse_csv(data)
        elif media_type in [
            "application/vnd.ms-excel",
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        ]:
            extracted_text = parse_excel(data)

        # 2. Performance optimization: Image-based OCR only if needed
        # We'll skip EasyOCR by default and let Gemini handle vision,
        # unless it's a very complex bank statement image.
        # For now, we only run it for images to provide context.
        ocr_context = ""
        if media_type.startswith("image/") and len(data) < 5 * 1024 * 1024:  # Only for reasonable size images
            try:
                # We do this in a thread to not block the event loop
                from PIL import Image
                import numpy as np

                img = Image.open(io.BytesIO(data))
                img_np = np.array(img)
                # detail=0 returns only text list
                loop = asyncio.get_event_loop()
                results = await loop.run_in_executor(None, lambda: self.reader.readtext(img_np, detail=0))
                ocr_context = " ".join(results)
            except Exception as e:
                print(f"EasyOCR error: {e}")

        # 3. Prepare Gemini request
        content_items = ["Extract data from this document."]

        if extracted_text:
            content_items.append(f"EXTRACTED TEXT CONTENT:\n{extracted_text}")
        if ocr_context:
            content_items.append(f"OCR CONTEXT (HEURISTIC):\n{ocr_context}")

        # Add the media part (vision)
        # Gemini 1.5+ handles PDF directly too, but we provide text for better accuracy
        content_items.append(types.Part.from_bytes(data=data, mime_type=media_type))

        try:
            response = self.client.models.generate_content(
                model=self.model_name,
                contents=content_items,
                config=types.GenerateContentConfig(
                    system_instruction=self.get_system_prompt(),
                    response_mime_type="application/json",
                    response_schema=UnifiedExtraction,
                    temperature=0.0,
                ),
            )

            raw_result = self._parse_json(response.text)

            # Map UnifiedExtraction back to the format Rust expects
            doc_type = raw_result.get("doc_type", "GENERIC")
            confidence = raw_result.get("confidence_score", 1.0)

            final_data = {}
            if doc_type == "GPAY":
                final_data = raw_result.get("gpay_data") or {}
            elif doc_type == "BANK_STATEMENT":
                # Rust expects a nested bank_data or the flat result?
                # Based on crates/db/src/lib.rs, it expects a structure that matches BankExtractionResult
                # which has bank_data field.
                final_data = {
                    "raw_text": extracted_text or ocr_context or "Extracted from vision",
                    "doc_type": "bank_statement",
                    "confidence_score": confidence,
                    "bank_data": raw_result.get("bank_data") or {},
                }
            else:
                final_data = raw_result.get("generic_data") or {}

            # Ensure confidence score is present in the data too
            if isinstance(final_data, dict):
                final_data["confidence_score"] = confidence
                if not final_data.get("raw_text"):
                    final_data["raw_text"] = extracted_text or ocr_context or "Extracted from vision"

            return {"doc_type": doc_type, "data": final_data}

        except Exception as e:
            print(f"Extraction error: {e}")
            raise e

    def _parse_json(self, text: str) -> dict:
        try:
            return json.loads(text)
        except json.JSONDecodeError:
            text = text.strip()
            if text.startswith("```"):
                text = text.split("```")[1]
                if text.startswith("json"):
                    text = text[4:]
            return json.loads(text.strip())
