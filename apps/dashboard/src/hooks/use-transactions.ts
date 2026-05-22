import type { DashboardSummary, Transaction, TransactionWithDetail } from "@expent/types";
import { toast } from "@expent/ui/components/goey-toaster";
import { useLiveQuery } from "@tanstack/react-db";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api-client";
import { useSession } from "@/lib/auth-client";
import { useCategories } from "./use-categories";
import { useWallets } from "./use-wallets";
import { db } from "@/lib/db";
import { aggregateTransactionsWasm, generateDashboardSummaryWasm } from "@expent/wasm";

export function useTransactions(params: { limit?: number; offset?: number } = {}) {
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
    mutationFn: async ({ id, data }: { id: string; data: Partial<TransactionWithDetail> }) => {
      // 1. Send to server
      const updatedTxn = await api.patch<Transaction>(`/api/transactions/${id}`, data);

      // 2. Update local DB
      db.transactions.update(id, (draft) => {
        Object.assign(draft, updatedTxn);
      });

      return updatedTxn;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["wallets"] });
      queryClient.invalidateQueries({ queryKey: ["transaction-summary"] });
      toast.success("Transaction updated");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  const deleteMutation = useMutation({
    mutationFn: async (id: string) => {
      // 1. Send to server
      await api.delete(`/api/transactions/${id}`);

      // 2. Delete from local DB
      db.transactions.delete(id);
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["wallets"] });
      queryClient.invalidateQueries({ queryKey: ["transaction-summary"] });
      toast.success("Transaction deleted");
    },
    onError: (error: Error) => toast.error(error.message),
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

  const { data: localSummary, isLoading: isComputing } = useQuery({
    queryKey: ["local-summary", transactions?.length, wallets?.length, categories?.length],
    queryFn: async () => {
      if (!transactions || transactions.length === 0) return null;
      return generateDashboardSummaryWasm(transactions, wallets || [], categories || []);
    },
    enabled: !!transactions && transactions.length > 0,
  });

  return {
    summary: localSummary,
    isLoading: isTxnsLoading || isWalletsLoading || isCatsLoading || isComputing,
  };
}
