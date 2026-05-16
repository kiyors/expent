import type { Wallet } from "@expent/types";
import { toast } from "@expent/ui/components/goey-toaster";
import { useLiveQuery } from "@tanstack/react-db";
import { useMutation } from "@tanstack/react-query";
import { api } from "@/lib/api-client";
import { useSession } from "@/lib/auth-client";
import { db } from "@/lib/db";

export function useWallets() {
  const session = useSession();

  // Use TanStack DB for the live query
  const query = useLiveQuery((q) => q.from({ wallets: db.wallets }), [session.data]);

  const createMutation = useMutation({
    mutationFn: async (data: { name: string; type: string; initial_balance: number }) => {
      // 1. Send to server
      const newWallet = await api.post<Wallet>("/api/wallets", data);

      // 2. Insert into local DB (TanStack DB will react immediately)
      db.wallets.insert(newWallet);

      return newWallet;
    },
    onSuccess: () => {
      toast.success("Wallet created");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  const updateMutation = useMutation({
    mutationFn: async ({ id, data }: { id: string; data: Partial<Wallet> }) => {
      // 1. Send to server
      const updatedWallet = await api.put<Wallet>(`/api/wallets/${id}`, data);

      // 2. Update local DB
      db.wallets.update(id, (draft) => {
        Object.assign(draft, updatedWallet);
      });

      return updatedWallet;
    },
    onSuccess: () => {
      toast.success("Wallet updated");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  const deleteMutation = useMutation({
    mutationFn: async (id: string) => {
      // 1. Send to server
      await api.delete(`/api/wallets/${id}`);

      // 2. Delete from local DB
      db.wallets.delete(id);
    },
    onSuccess: () => {
      toast.success("Wallet deleted");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  return {
    wallets: query.data as unknown as Wallet[],
    isLoading: query.isLoading,
    error: query.isError ? "Error loading wallets" : null,
    createMutation,
    updateMutation,
    deleteMutation,
  };
}
