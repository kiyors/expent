import type {
  DashboardSummary,
  PaginationParams,
  Transaction,
  TransactionWithDetail,
  UpdateTransactionRequest,
  ValidationResult,
} from "@expent/types";
import { toast } from "@expent/ui/components/goey-toaster";
import { useLiveQuery } from "@tanstack/react-db";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api-client";
import { useSession } from "@/lib/auth-client";
import { useCategories } from "./use-categories";
import { useWallets } from "./use-wallets";
import { db } from "@/lib/db";
import {
  aggregateTransactionsWasm,
  generateDashboardSummaryWasm,
  validateTransactionWasm,
  useWasmWorker,
} from "@expent/wasm";

export function useTransactions(params: PaginationParams = {}) {
  const session = useSession();
  const queryClient = useQueryClient();

  // Use TanStack DB for the live query
  const query = useLiveQuery(
    (q) => {
      let query = q.from({ transactions: db.transactions }).orderBy(({ transactions }) => transactions.date, "desc");
      if (params.limit) query = query.limit(params.limit);
      if (params.offset) query = query.offset(params.offset);
      return query;
    },
    [params.limit, params.offset, session.data],
  );

  const updateMutation = useMutation({
    mutationFn: async ({ id, data }: { id: string; data: UpdateTransactionRequest }) => {
      // 0. Shared WASM Validation
      if (data.amount || data.purpose_tag) {
        const currentTxn = db.transactions.get(id);
        const amount = data.amount?.toString() || currentTxn?.amount || "0";
        const purpose = data.purpose_tag || currentTxn?.purpose_tag || "";

        const result = (await validateTransactionWasm(amount, purpose)) as unknown as ValidationResult;
        if (!result.is_valid) {
          throw new Error(result.errors.map((e) => `${e.field}: ${e.message}`).join(", "));
        }
      }

      return api.patch<Transaction, UpdateTransactionRequest>(`/api/transactions/${id}`, data);
    },
    onMutate: async ({ id, data }) => {
      // 1. Cancel outgoing refetches
      await queryClient.cancelQueries({ queryKey: ["transaction-summary"] });

      // 2. Snapshot the previous state (TanStack DB handles its own snapshots usually, but we might want to be safe)
      const previousTxn = db.transactions.get(id);

      // 3. Optimistically update local DB
      db.transactions.update(id, (draft) => {
        Object.assign(draft, data);
      });

      return { previousTxn };
    },
    onError: (err, { id }, context) => {
      // Rollback on error
      if (context?.previousTxn) {
        db.transactions.update(id, (draft) => {
          Object.assign(draft, context.previousTxn);
        });
      }
      toast.error(err.message);
    },
    onSuccess: (updatedTxn, { id }) => {
      // Ensure local DB is in sync with server's final version
      db.transactions.update(id, (draft) => {
        Object.assign(draft, updatedTxn);
      });
      queryClient.invalidateQueries({ queryKey: ["wallets"] });
      queryClient.invalidateQueries({ queryKey: ["transaction-summary"] });
      toast.success("Transaction updated");
    },
  });

  const deleteMutation = useMutation({
    mutationFn: async (id: string) => {
      await api.delete(`/api/transactions/${id}`);
    },
    onMutate: async (id) => {
      await queryClient.cancelQueries({ queryKey: ["transaction-summary"] });
      const previousTxn = db.transactions.get(id);

      // Optimistically delete
      db.transactions.delete(id);

      return { previousTxn };
    },
    onError: (err, id, context) => {
      // Rollback
      if (context?.previousTxn) {
        db.transactions.insert(context.previousTxn);
      }
      toast.error(err.message);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["wallets"] });
      queryClient.invalidateQueries({ queryKey: ["transaction-summary"] });
      toast.success("Transaction deleted");
    },
  });

  return {
    transactions: query.data as unknown as TransactionWithDetail[],
    totalCount: (query as any).totalCount || 0, // TanStack DB handles total count in meta
    isLoading: query.isLoading,
    isFetching: query.isLoading, // In DB mode, loading is fetching
    error: query.isError ? "Error loading transactions" : null,
    updateMutation,
    deleteMutation,
  };
}

export function useTransactionSummary() {
  const session = useSession();

  const query = useQuery({
    queryKey: ["transaction-summary"],
    queryFn: () => api.get<DashboardSummary>("/api/transactions/summary"),
    enabled: !!session.data,
    staleTime: 1000 * 60 * 5, // 5 minutes
  });

  return {
    summary: query.data,
    isLoading: query.isLoading,
    isFetching: query.isFetching,
    error: query.error,
    refetch: query.refetch,
  };
}

export function useLocalSummary() {
  const { transactions, isLoading: isTxnsLoading } = useTransactions({ limit: 1000 });
  const { wallets, isLoading: isWalletsLoading } = useWallets();
  const { categories, isLoading: isCatsLoading } = useCategories();
  const { runTask } = useWasmWorker();

  const { data: localSummary, isLoading: isComputing } = useQuery({
    queryKey: ["local-summary", transactions?.length, wallets?.length, categories?.length],
    queryFn: async () => {
      if (!transactions || transactions.length === 0) return null;
      return runTask<DashboardSummary>("GENERATE_SUMMARY", {
        transactions,
        wallets: wallets || [],
        categories: categories || [],
      });
    },
    enabled: !!transactions && transactions.length > 0,
  });

  return {
    summary: localSummary,
    isLoading: isTxnsLoading || isWalletsLoading || isCatsLoading || isComputing,
  };
}
