import { toast } from "@expent/ui/components/goey-toaster";

/**
 * Validates a PDF file's page count using WebAssembly (mupdf).
 * Returns true if valid, false if invalid (or if validation should be skipped).
 * @param file The PDF file to validate
 * @param maxPages Maximum allowed pages
 */
export async function validatePdfPageCount(file: File, maxPages: number = 5): Promise<boolean> {
  if (file.type !== "application/pdf") return true;

  try {
    const mupdf = await import("mupdf");
    const pdfData = await file.arrayBuffer();
    const doc = mupdf.Document.openDocument(pdfData, "application/pdf");
    const numPages = doc.countPages();

    if (numPages > maxPages) {
      toast.error(`PDF too long (${numPages} pages). Max ${maxPages} pages allowed for OCR.`);
      return false;
    }
    return true;
  } catch (err) {
    console.warn("WASM PDF validation skipped:", err);
    // Return true to avoid blocking users if WASM fails to load or run
    return true;
  }
}
