import pytest
import sys
import os
import asyncio
from unittest.mock import MagicMock, AsyncMock, patch

# Mock heavy/missing dependencies
sys.modules["google"] = MagicMock()
sys.modules["google.genai"] = MagicMock()
sys.modules["google.genai.types"] = MagicMock()
sys.modules["easyocr"] = MagicMock()
sys.modules["fitz"] = MagicMock()
sys.modules["pdfplumber"] = MagicMock()
sys.modules["PIL"] = MagicMock()
sys.modules["numpy"] = MagicMock()

# Mock routers to avoid import issues
mock_routers = MagicMock()
sys.modules["routers"] = mock_routers
sys.modules["routers.gpay"] = MagicMock()
sys.modules["routers.gpay.prompts"] = MagicMock()
sys.modules["routers.gpay.schemas"] = MagicMock()
sys.modules["routers.generic_receipt"] = MagicMock()
sys.modules["routers.generic_receipt.prompts"] = MagicMock()
sys.modules["routers.generic_receipt.schemas"] = MagicMock()

# Add parent directory to path to allow absolute imports
sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), "../..")))

with patch.dict(os.environ, {"GOOGLE_API_KEY": "fake_key"}):
    from apps.ocr.core import OCREngine

def run_async(coro):
    return asyncio.run(coro)

def test_classify_image_gpay():
    engine = OCREngine(api_key="fake_key")

    # Mock the Gemini client response
    mock_response = MagicMock()
    mock_response.text = "This is a GPAY screenshot"
    engine.client.models.generate_content.return_value = mock_response

    result = run_async(engine.classify_image(b"fake_data", "image/png"))

    assert result == "GPAY"
    engine.client.models.generate_content.assert_called_once()

def test_classify_image_generic():
    engine = OCREngine(api_key="fake_key")

    # Mock the Gemini client response
    mock_response = MagicMock()
    mock_response.text = "This is a GENERIC receipt"
    # Ensure it's not poisoned by previous tests if any (though pytest usually isolates)
    engine.client.models.generate_content.reset_mock()
    engine.client.models.generate_content.return_value = mock_response

    result = run_async(engine.classify_image(b"fake_data", "image/png"))

    assert result == "GENERIC"
    engine.client.models.generate_content.assert_called_once()

def test_classify_image_exception_generic():
    engine = OCREngine(api_key="fake_key")

    # Mock the Gemini client to throw a generic exception
    engine.client.models.generate_content.side_effect = Exception("Some random error")

    result = run_async(engine.classify_image(b"fake_data", "image/png"))

    # Should fallback to GENERIC
    assert result == "GENERIC"

def test_classify_image_quota_error_429():
    engine = OCREngine(api_key="fake_key")

    # Mock the Gemini client to throw a 429 error
    error_msg = "429 Resource has been exhausted (e.g. check quota)."
    engine.client.models.generate_content.side_effect = Exception(error_msg)

    with pytest.raises(Exception) as excinfo:
        run_async(engine.classify_image(b"fake_data", "image/png"))

    assert error_msg in str(excinfo.value)

def test_classify_image_quota_error_word():
    engine = OCREngine(api_key="fake_key")

    # Mock the Gemini client to throw a quota error
    error_msg = "Your quota is exceeded."
    engine.client.models.generate_content.side_effect = Exception(error_msg)

    with pytest.raises(Exception) as excinfo:
        run_async(engine.classify_image(b"fake_data", "image/png"))

    assert error_msg in str(excinfo.value)
