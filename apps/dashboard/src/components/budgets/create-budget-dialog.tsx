"use client";

import type { BudgetPeriod } from "@expent/types";
import { Button } from "@expent/ui/components/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@expent/ui/components/dialog";
import { toast } from "@expent/ui/components/goey-toaster";
import { Input } from "@expent/ui/components/input";
import { Label } from "@expent/ui/components/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@expent/ui/components/select";
import * as React from "react";
import { useBudgets } from "@/hooks/use-budgets";
import { useCategories } from "@/hooks/use-categories";

interface CreateBudgetDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function CreateBudgetDialog({ open, onOpenChange }: CreateBudgetDialogProps) {
  const [categoryId, setCategoryId] = React.useState<string>("all");
  const [amount, setAmount] = React.useState("");
  const [period, setPeriod] = React.useState<BudgetPeriod>("MONTHLY");

  const { createMutation } = useBudgets();
  const { categories } = useCategories();

  React.useEffect(() => {
    if (open) {
      setCategoryId("all");
      setAmount("");
      setPeriod("MONTHLY");
    }
  }, [open]);

  const handleSubmit = () => {
    if (!amount || isNaN(Number(amount)) || Number(amount) <= 0) {
      toast.error("Please enter a valid amount");
      return;
    }

    createMutation.mutate(
      {
        category_id: categoryId === "all" ? undefined : categoryId,
        amount,
        period,
      },
      {
        onSuccess: () => {
          toast.success("Budget set!");
          onOpenChange(false);
        },
        onError: (err) => {
          toast.error(err.message || "Failed to set budget");
        },
      },
    );
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Set Budget</DialogTitle>
          <DialogDescription>Define a spending limit for a specific category or overall.</DialogDescription>
        </DialogHeader>

        <div className="grid gap-4 py-4">
          <div className="grid gap-2">
            <Label htmlFor="category">Category</Label>
            <Select value={categoryId} onValueChange={(v) => setCategoryId(v || "all")}>
              <SelectTrigger id="category">
                <SelectValue placeholder="Select Category" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Categories</SelectItem>
                {categories?.map((cat) => (
                  <SelectItem key={cat.id} value={cat.id}>
                    {cat.name}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          <div className="grid gap-2">
            <Label htmlFor="amount">Limit Amount</Label>
            <div className="relative">
              <span className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground text-sm">₹</span>
              <Input
                id="amount"
                className="pl-7"
                placeholder="0.00"
                type="number"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
              />
            </div>
          </div>

          <div className="grid gap-2">
            <Label htmlFor="period">Period</Label>
            <Select value={period} onValueChange={(v) => setPeriod((v as BudgetPeriod) || "MONTHLY")}>
              <SelectTrigger id="period">
                <SelectValue placeholder="Select Period" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="WEEKLY">Weekly</SelectItem>
                <SelectItem value="MONTHLY">Monthly</SelectItem>
                <SelectItem value="YEARLY">Yearly</SelectItem>
              </SelectContent>
            </Select>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={createMutation.isPending}>
            {createMutation.isPending ? "Setting..." : "Set Budget"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
