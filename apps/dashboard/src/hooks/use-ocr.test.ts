import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi, beforeEach } from "vitest";
import { useOcrUpload } from "./use-ocr";
import { toast } from "@expent/ui/components/goey-toaster";
import { api } from "@/lib/api-client";

// Mock dependencies
vi.mock("@expent/ui/components/goey-toaster", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

vi.mock("@tanstack/react-query", () => ({
  useQueryClient: () => ({
    invalidateQueries: vi.fn(),
  }),
}));

vi.mock("@/lib/api-client", () => ({
  api: {
    get: vi.fn(),
    post: vi.fn(),
  },
}));

// Mock mupdf
vi.mock("mupdf", () => ({
  Document: {
    openDocument: vi.fn(),
  },
}));

// Helper to create a mock File
const createMockFile = (name: string, type: string) => {
  const file = new File([""], name, { type });
  Object.defineProperty(file, "arrayBuffer", {
    value: async () => new ArrayBuffer(0),
  });
  return file;
};

describe("useOcrUpload", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Global fetch mock
    global.fetch = vi.fn();
    // The shared SSE stub in setup.ts checks globalThis.__MOCK_SSE_PAYLOAD__
    // before falling back to its default; reset it between tests so payloads
    // don't bleed across cases.
    (globalThis as any).__MOCK_SSE_PAYLOAD__ = undefined;
  });

  it("should block PDF upload if it has more than 5 pages", async () => {
    const mockFile = createMockFile("large.pdf", "application/pdf");

    const mupdf = await import("mupdf");
    (mupdf.Document.openDocument as any).mockReturnValue({
      countPages: () => 10,
    });

    const { result } = renderHook(() => useOcrUpload());

    await act(async () => {
      const uploadResult = await result.current.uploadFile(mockFile);
      expect(uploadResult).toBeNull();
    });

    expect(toast.error).toHaveBeenCalledWith(expect.stringContaining("PDF too long (10 pages)"));
    expect(global.fetch).not.toHaveBeenCalled();
  });

  it("should allow PDF upload if it has 5 or fewer pages", async () => {
    const mockFile = createMockFile("small.pdf", "application/pdf");

    // Mock mupdf to return 3 pages
    const mupdf = await import("mupdf");
    (mupdf.Document.openDocument as any).mockReturnValue({
      countPages: () => 3,
    });

    // Mock successful upload and process
    (global.fetch as any).mockResolvedValue({
      ok: true,
      json: async () => ({ key: "file-key" }),
    });
    (api.post as any).mockResolvedValue({ job_id: "job-123" });
    (api.get as any).mockResolvedValue({
      status: "COMPLETED",
      processed_data: { doc_type: "GPAY", data: {} },
    });
    // The SSE stub fires this payload when waitForJobCompletion subscribes;
    // its job_id must match the one api.post just returned or the hook ignores
    // the message and times out.
    (globalThis as any).__MOCK_SSE_PAYLOAD__ = {
      job_id: "job-123",
      status: "COMPLETED",
      processed_data: { doc_type: "GPAY", data: {} },
    };

    const { result } = renderHook(() => useOcrUpload());

    await act(async () => {
      const uploadResult = await result.current.uploadFile(mockFile);
      expect(uploadResult).not.toBeNull();
    });

    expect(global.fetch).toHaveBeenCalled();
    expect(api.post).toHaveBeenCalledWith("/api/ocr/process", { key: "file-key" });
  });

  it("should allow image upload without page count check", async () => {
    const mockFile = createMockFile("receipt.png", "image/png");

    // Mock successful upload and process
    (global.fetch as any).mockResolvedValue({
      ok: true,
      json: async () => ({ key: "file-key" }),
    });
    (api.post as any).mockResolvedValue({ job_id: "job-123" });
    (api.get as any).mockResolvedValue({
      status: "COMPLETED",
      processed_data: { doc_type: "GPAY", data: {} },
    });
    (globalThis as any).__MOCK_SSE_PAYLOAD__ = {
      job_id: "job-123",
      status: "COMPLETED",
      processed_data: { doc_type: "GPAY", data: {} },
    };

    const { result } = renderHook(() => useOcrUpload());

    await act(async () => {
      const uploadResult = await result.current.uploadFile(mockFile);
      expect(uploadResult).not.toBeNull();
    });

    expect(global.fetch).toHaveBeenCalled();
  });
});
