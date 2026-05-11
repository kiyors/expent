import { toast } from "@expent/ui/components/goey-toaster";
import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { api } from "@/lib/api-client";
import { useTransactionSummary, useTransactions } from "./use-transactions";

// Mock dependencies
vi.mock("@/lib/api-client", () => ({
  api: {
    get: vi.fn(),
    post: vi.fn(),
    put: vi.fn(),
    patch: vi.fn(),
    delete: vi.fn(),
  },
}));
vi.mock("@/lib/auth-client", () => ({
  useSession: () => ({ data: { user: { id: "test-user" } } }),
}));
vi.mock("@tanstack/react-query", () => ({
  useMutation: vi.fn(({ mutationFn, onSuccess, onError }) => ({
    mutateAsync: async (variables: any) => {
      try {
        const result = await mutationFn(variables);
        if (onSuccess) onSuccess(result, variables);
        return result;
      } catch (error) {
        if (onError) onError(error);
        throw error;
      }
    },
    isLoading: false,
  })),
  useQuery: vi.fn(({ queryFn }) => {
    queryFn(); // Execute it to verify it was called
    return { data: null, isLoading: false };
  }),
  useQueryClient: () => ({
    invalidateQueries: vi.fn(),
  }),
}));
vi.mock("@tanstack/react-db", () => ({
  useLiveQuery: vi.fn(() => ({ data: [], isLoading: false })),
}));
vi.mock("@/lib/db", () => ({
  db: {
    transactions: {
      update: vi.fn(),
      delete: vi.fn(),
    },
  },
}));

describe("useTransactions", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should handle update transaction success", async () => {
    const mockTxn = { id: "1", amount: "100" };
    (api.patch as any).mockResolvedValue(mockTxn);

    const { result } = renderHook(() => useTransactions());

    await act(async () => {
      await result.current.updateMutation.mutateAsync({ id: "1", data: { amount: "100" } });
    });

    expect(api.patch).toHaveBeenCalledWith("/api/transactions/1", { amount: "100" });
    expect(toast.success).toHaveBeenCalledWith("Transaction updated");
  });

  it("should handle update transaction error", async () => {
    const error = new Error("API Error");
    (api.patch as any).mockRejectedValue(error);

    const { result } = renderHook(() => useTransactions());

    try {
      await act(async () => {
        await result.current.updateMutation.mutateAsync({ id: "1", data: { amount: "100" } });
      });
    } catch (e) {
      // Expected
    }

    expect(toast.error).toHaveBeenCalledWith("API Error");
  });

  it("should handle delete transaction success", async () => {
    (api.delete as any).mockResolvedValue({});

    const { result } = renderHook(() => useTransactions());

    await act(async () => {
      await result.current.deleteMutation.mutateAsync("1");
    });

    expect(api.delete).toHaveBeenCalledWith("/api/transactions/1");
    expect(toast.success).toHaveBeenCalledWith("Transaction deleted");
  });
});

describe("useTransactionSummary", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should fetch summary success", async () => {
    const mockSummary = { total_balance: 100 };
    (api.get as any).mockResolvedValue(mockSummary);

    renderHook(() => useTransactionSummary());

    expect(api.get).toHaveBeenCalledWith("/api/transactions/summary");
  });
});
