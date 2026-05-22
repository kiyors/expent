"use client";

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
import { useWallets } from "@/hooks/use-wallets";

interface CreateWalletDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onCreated?: (walletId: string) => void;
}

export function CreateWalletDialog({ open, onOpenChange, onCreated }: CreateWalletDialogProps) {
  const [newName, setNewName] = React.useState("");
  const [newType, setNewType] = React.useState("CASH");
  const [newBalance, setNewBalance] = React.useState("0");

  const { createMutation } = useWallets();

  React.useEffect(() => {
    if (open) {
      setNewName("");
      setNewType("CASH");
      setNewBalance("0");
    }
  }, [open]);

  const handleCreate = () => {
    if (!newName.trim()) {
      toast.error("Wallet name is required");
      return;
    }

    createMutation.mutate(
      {
        name: newName.trim(),
        type: newType,
        initial_balance: parseFloat(newBalance) || 0,
      },
      {
        onSuccess: (data: { id: string }) => {
          toast.success("Wallet created!");
          onOpenChange(false);
          if (onCreated && data?.id) {
            onCreated(data.id);
          }
        },
        onError: (err: Error) => {
          toast.error(err.message || "Failed to create wallet");
        },
      },
    );
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[420px]">
        <DialogHeader>
          <DialogTitle>Create Wallet</DialogTitle>
          <DialogDescription>Add a new bank account, credit card, or cash wallet.</DialogDescription>
        </DialogHeader>

        <div className="grid gap-4 py-4">
          <div className="grid gap-2">
            <Label htmlFor="wallet-name">Wallet Name</Label>
            <Input
              id="wallet-name"
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              placeholder="e.g. HDFC Bank, My Credit Card"
              autoComplete="off"
            />
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div className="grid gap-2">
              <Label htmlFor="wallet-type">Type</Label>
              <Select value={newType} onValueChange={(val) => setNewType(val || "CASH")}>
                <SelectTrigger id="wallet-type">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="CASH">Cash</SelectItem>
                  <SelectItem value="BANK">Bank Account</SelectItem>
                  <SelectItem value="CREDIT_CARD">Credit Card</SelectItem>
                  <SelectItem value="UPI">UPI Wallet</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className="grid gap-2">
              <Label htmlFor="wallet-balance">Initial Balance (₹)</Label>
              <Input
                id="wallet-balance"
                type="number"
                step="0.01"
                value={newBalance}
                onChange={(e) => setNewBalance(e.target.value)}
              />
            </div>
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleCreate} disabled={!newName.trim() || createMutation.isPending}>
            {createMutation.isPending ? "Creating..." : "Create Wallet"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
