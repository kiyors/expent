"use client";

import { Button } from "@expent/ui/components/button";
import { toast } from "@expent/ui/components/goey-toaster";
import { Progress, ProgressIndicator, ProgressTrack } from "@expent/ui/components/progress";
import { Separator } from "@expent/ui/components/separator";
import { cn } from "@expent/ui/lib/utils";
import { PlusIcon, TargetIcon, Trash2Icon } from "lucide-react";
import * as React from "react";
import { CreateBudgetDialog } from "@/components/budgets/create-budget-dialog";
import { useBudgets } from "@/hooks/use-budgets";

export default function SettingsBudgetsPage() {
  const { health, isLoading, deleteMutation } = useBudgets();
  const [createOpen, setCreateOpen] = React.useState(false);

  const handleDelete = (id: string) => {
    deleteMutation.mutate(id, {
      onSuccess: () => toast.success(`Budget deleted.`),
      onError: (err: any) => toast.error(err.message || "Failed to delete budget"),
    });
  };

  const formatCurrency = (amount: string) => {
    return new Intl.NumberFormat("en-IN", {
      style: "currency",
      currency: "INR",
      maximumFractionDigits: 0,
    }).format(Number(amount));
  };

  return (
    <div className="space-y-6 w-full max-w-2xl">
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-medium">Budgets</h3>
          <p className="text-sm text-muted-foreground">Set spending limits for your categories.</p>
        </div>
        <Button size="sm" onClick={() => setCreateOpen(true)}>
          <PlusIcon className="mr-2 h-4 w-4" /> New Budget
        </Button>
      </div>
      <Separator />

      {isLoading ? (
        <div className="space-y-3">
          {[1, 2, 3].map((i) => (
            <div key={i} className="h-24 animate-pulse rounded-lg bg-muted/50" />
          ))}
        </div>
      ) : (
        <div className="space-y-4">
          {!health || health.length === 0 ? (
            <div className="flex flex-col items-center justify-center rounded-lg border border-dashed p-8 text-center">
              <TargetIcon className="h-8 w-8 text-muted-foreground/40 mb-3" />
              <p className="text-sm text-muted-foreground">No budgets set yet.</p>
              <p className="text-xs text-muted-foreground mt-1">Set a budget to keep your spending in check.</p>
              <Button size="sm" variant="outline" className="mt-4" onClick={() => setCreateOpen(true)}>
                <PlusIcon className="mr-2 h-4 w-4" /> Create Budget
              </Button>
            </div>
          ) : (
            health.map((b) => {
              const percentage = Number(b.percentage_consumed);
              const isOver = percentage > 100;
              const isWarning = percentage > 85;

              return (
                <div
                  key={b.budget_id}
                  className="flex flex-col gap-4 rounded-lg border p-4 group hover:border-primary/30 transition-colors"
                >
                  <div className="flex items-center justify-between">
                    <div>
                      <p className="text-sm font-medium">{b.category_name}</p>
                      <p className="text-xs text-muted-foreground uppercase tracking-tight">
                        {b.period} · {formatCurrency(b.limit_amount)} limit
                      </p>
                    </div>
                    <Button
                      variant="ghost"
                      size="icon-xs"
                      className="h-8 w-8 opacity-0 group-hover:opacity-100 transition-opacity text-muted-foreground hover:text-destructive"
                      onClick={() => handleDelete(b.budget_id)}
                      disabled={deleteMutation.isPending}
                    >
                      <Trash2Icon className="h-4 w-4" />
                    </Button>
                  </div>
                  <div className="space-y-2">
                    <div className="flex items-center justify-between text-xs">
                      <span className="text-muted-foreground">Spent: {formatCurrency(b.spent_amount)}</span>
                      <span className={cn("font-medium", isOver ? "text-destructive" : "text-muted-foreground")}>
                        {isOver ? "Over by " : "Remaining: "}
                        {formatCurrency(Math.abs(Number(b.remaining_amount)).toString())}
                      </span>
                    </div>
                    <Progress value={Math.min(percentage, 100)}>
                      <ProgressTrack>
                        <ProgressIndicator
                          className={cn(isOver ? "bg-destructive" : isWarning ? "bg-amber-500" : "bg-primary")}
                        />
                      </ProgressTrack>
                    </Progress>
                    <div className="flex justify-end">
                      <span
                        className={cn(
                          "text-[10px] uppercase font-bold",
                          isOver ? "text-destructive" : isWarning ? "text-amber-500" : "text-muted-foreground/60",
                        )}
                      >
                        {percentage.toFixed(0)}% consumed
                      </span>
                    </div>
                  </div>
                </div>
              );
            })
          )}
        </div>
      )}

      <CreateBudgetDialog open={createOpen} onOpenChange={setCreateOpen} />
    </div>
  );
}
