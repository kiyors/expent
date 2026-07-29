import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api } from "@/lib/ApiClient";

export function useDemoData() {
  const queryClient = useQueryClient();

  const { data: status, isLoading } = useQuery({
    queryKey: ["demo-status"],
    queryFn: async () => {
      const res = await api.get<{ active: boolean }>("/api/demo/status");
      return res;
    },
  });

  const seedMutation = useMutation({
    mutationFn: async () => {
      await api.post("/api/demo/seed");
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["demo-status"] });
      void queryClient.invalidateQueries({ queryKey: ["transactions"] });
      void queryClient.invalidateQueries({ queryKey: ["wallets"] });
      void queryClient.invalidateQueries({ queryKey: ["categories"] });
      void queryClient.invalidateQueries({ queryKey: ["budgets"] });
      void queryClient.invalidateQueries({ queryKey: ["contacts"] });
      void queryClient.invalidateQueries({ queryKey: ["transaction-summary"] });
    },
  });

  const clearMutation = useMutation({
    mutationFn: async () => {
      await api.post("/api/demo/clear");
    },
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["demo-status"] });
      void queryClient.invalidateQueries({ queryKey: ["transactions"] });
      void queryClient.invalidateQueries({ queryKey: ["wallets"] });
      void queryClient.invalidateQueries({ queryKey: ["categories"] });
      void queryClient.invalidateQueries({ queryKey: ["budgets"] });
      void queryClient.invalidateQueries({ queryKey: ["contacts"] });
      void queryClient.invalidateQueries({ queryKey: ["transaction-summary"] });
    },
  });

  return {
    isActive: status?.active ?? false,
    isLoading,
    seedMutation,
    clearMutation,
  };
}
