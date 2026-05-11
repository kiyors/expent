import base64
import fitz  # PyMuPDF
import csv
import io
import pdfplumber
import pandas as pd


def get_media_type(filename: str) -> str:
    ext = filename.split(".")[-1].lower() if filename else "png"
    return {
        "png": "image/png",
        "jpg": "image/jpeg",
        "jpeg": "image/jpeg",
        "webp": "image/webp",
        "pdf": "application/pdf",
        "csv": "text/csv",
        "xls": "application/vnd.ms-excel",
        "xlsx": "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    }.get(ext, "image/png")


def to_base64(data: bytes) -> str:
    return base64.standard_b64encode(data).decode()


def rasterize_pdf_page(pdf_bytes: bytes, page_num: int = 0, dpi: int = 150) -> bytes:
    """Convert a single PDF page into PNG bytes for vision processing."""
    doc = fitz.open(stream=pdf_bytes, filetype="pdf")
    page = doc.load_page(page_num)
    pix = page.get_pixmap(dpi=dpi)
    return pix.tobytes("png")


def extract_pdf_text(pdf_bytes: bytes) -> str:
    """Extract text from PDF using pdfplumber."""
    text = ""
    try:
        with pdfplumber.open(io.BytesIO(pdf_bytes)) as pdf:
            if pdf.is_encrypted:
                return "ERROR: PDF is password protected. Please remove the password before uploading."
            for page in pdf.pages:
                page_text = page.extract_text()
                if page_text:
                    text += page_text + "\n"
    except Exception as e:
        print(f"PDF extraction error: {e}")
        return f"ERROR: Failed to parse PDF: {str(e)}"
    return text.strip()


def parse_csv(csv_bytes: bytes) -> str:
    """Parse CSV bytes into a string representation for LLM."""
    try:
        content = csv_bytes.decode("utf-8")
        return content
    except UnicodeDecodeError:
        try:
            content = csv_bytes.decode("latin-1")
            return content
        except Exception:
            return "Error decoding CSV"


def parse_excel(excel_bytes: bytes) -> str:
    """Parse Excel bytes into a string representation (CSV-like) for LLM."""
    try:
        df = pd.read_excel(io.BytesIO(excel_bytes))
        return df.to_csv(index=False)
    except Exception as e:
        return f"Error parsing Excel: {str(e)}"
