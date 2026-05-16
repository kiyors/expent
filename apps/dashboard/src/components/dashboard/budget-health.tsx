"use client";

import { Button } from "@expent/ui/components/button";
import { Progress, ProgressIndicator, ProgressTrack } from "@expent/ui/components/progress";
import { cn } from "@expent/ui/lib/utils";
import { TargetIcon } from "lucide-react";
import { useRouter } from "next/navigation";
import { useTransition } from "react";
import { useBudgets } from "@/hooks/use-budgets";

export function BudgetHealthWidget() {
  const { health, isLoading } = useBudgets();
  const router = useRouter();
  const [_isPending, startTransition] = useTransition();

  if (isLoading) {
    return (
      <div className="space-y-4 animate-pulse">
        {[1, 2, 3].map((i) => (
          <div key={i} className="space-y-2">
            <div className="flex justify-between">
              <div className="h-3 w-20 bg-muted rounded" />
              <div className="h-3 w-8 bg-muted rounded" />
            </div>
            <div className="h-1.5 w-full bg-muted rounded" />
          </div>
        ))}
      </div>
    );
  }

  if (!health || health.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-8 text-center">
        <TargetIcon className="h-8 w-8 text-muted-foreground/30 mb-2" />
        <p className="text-sm text-muted-foreground mb-4">No budgets set</p>
        <Button size="sm" variant="outline" onClick={() => startTransition(() => router.push("/settings/budgets"))}>
          Setup Budgets
        </Button>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="space-y-4">
        {health.slice(0, 4).map((b) => {
          const percentage = Number(b.percentage_consumed);
          const isOver = percentage > 100;
          const isWarning = percentage > 85;

          return (
            <div key={b.budget_id} className="space-y-1.5">
              <div className="flex items-center justify-between text-xs">
                <span className="font-medium truncate max-w-[150px]">{b.category_name}</span>
                <span
                  className={cn(
                    "font-bold",
                    isOver ? "text-destructive" : isWarning ? "text-amber-500" : "text-primary",
                  )}
                >
                  {percentage.toFixed(0)}%
                </span>
              </div>
              <Progress value={Math.min(percentage, 100)}>
                <ProgressTrack className="h-1.5 bg-muted/50">
                  <ProgressIndicator
                    className={cn(isOver ? "bg-destructive" : isWarning ? "bg-amber-500" : "bg-primary")}
                  />
                </ProgressTrack>
              </Progress>
            </div>
          );
        })}
      </div>
      {health.length > 4 && (
        <Button
          variant="link"
          size="sm"
          className="w-full text-xs text-muted-foreground"
          onClick={() => startTransition(() => router.push("/settings/budgets"))}
        >
          View all budgets &rarr;
        </Button>
      )}
    </div>
  );
}
