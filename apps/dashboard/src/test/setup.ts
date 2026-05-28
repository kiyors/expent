import "@testing-library/jest-dom";
import { vi } from "vitest";

// Set environment variables for tests
process.env.NEXT_PUBLIC_API_BASE_URL = "http://localhost:8000";

// Mock global fetch
global.fetch = vi.fn();

// jsdom doesn't ship EventSource; hooks that subscribe to SSE (e.g. useOcrUpload's
// /api/ocr/stream listener) would throw `ReferenceError: EventSource is not
// defined` at module load. A no-op stub is enough for tests that don't
// actually exercise the streamed updates — production code only reads from
// addEventListener and close().
// Default payload tests get back when a hook subscribes to an EventSource and
// doesn't override globalThis.__MOCK_SSE_PAYLOAD__. Picked to match what the
// OCR upload flow expects (a single COMPLETED message ends the wait).
const DEFAULT_SSE_PAYLOAD = {
  status: "COMPLETED",
  job_id: "test-job",
  transaction_id: "test-txn",
  processed_data: { doc_type: "GPAY", data: {} },
};

class MockEventSource {
  static readonly CONNECTING = 0;
  static readonly OPEN = 1;
  static readonly CLOSED = 2;
  readyState = MockEventSource.CONNECTING;
  url: string;
  withCredentials = false;
  // Plain function types (no `this: EventSource`) so the mock can invoke the
  // handlers from within its own constructor without TS complaining that
  // MockEventSource isn't structurally assignable to EventSource.
  onopen: ((ev: Event) => unknown) | null = null;
  onmessage: ((ev: MessageEvent) => unknown) | null = null;
  onerror: ((ev: Event) => unknown) | null = null;
  constructor(url: string) {
    this.url = url;
    // Queue a default COMPLETED message on the next microtask so consumers
    // that block on `onmessage` resolve instead of hanging the suite. Tests
    // that need different behaviour can replace `global.EventSource` per-test.
    queueMicrotask(() => {
      if (this.readyState === MockEventSource.CLOSED) return;
      this.readyState = MockEventSource.OPEN;
      const payload = (globalThis as any).__MOCK_SSE_PAYLOAD__ ?? DEFAULT_SSE_PAYLOAD;
      this.onmessage?.(new MessageEvent("message", { data: JSON.stringify(payload) }));
    });
  }
  addEventListener = vi.fn();
  removeEventListener = vi.fn();
  dispatchEvent = vi.fn(() => true);
  close = vi.fn(() => {
    this.readyState = MockEventSource.CLOSED;
  });
}
// @ts-expect-error — minimal stub for jsdom; only the surface our hooks touch.
global.EventSource = MockEventSource;

// Mock toast
vi.mock("@expent/ui/components/goey-toaster", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

// Mock Next.js navigation if needed
vi.mock("next/navigation", () => ({
  useRouter: () => ({
    push: vi.fn(),
    replace: vi.fn(),
    prefetch: vi.fn(),
  }),
  useSearchParams: () => new URLSearchParams(),
  usePathname: () => "/",
}));
