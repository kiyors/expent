import { useQuery } from "@tanstack/react-query";
import { detectSubscriptionPatternsWasm } from "@/lib/wasm-utils";
import { useTransactions } from "./use-transactions";

/**
 * A hook that uses WASM to detect subscription patterns locally from transaction data.
 */
export function useLocalSubscriptionDetection() {
  const { transactions, isLoading } = useTransactions({ limit: 1000 });

  const { data: detectedSubscriptions, isLoading: isDetecting } = useQuery({
    queryKey: ["local-detected-subscriptions", transactions?.length],
    queryFn: async () => {
      if (!transactions || transactions.length < 2) return [];
      return detectSubscriptionPatternsWasm(transactions);
    },
    enabled: !!transactions && transactions.length >= 2,
  });

  return {
    detectedSubscriptions,
    isLoading: isLoading || isDetecting,
  };
}
