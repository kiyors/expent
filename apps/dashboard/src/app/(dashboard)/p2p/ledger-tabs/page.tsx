"use client";

import type { LedgerTab } from "@expent/types";
import { Badge } from "@expent/ui/components/badge";
import { Button } from "@expent/ui/components/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@expent/ui/components/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@expent/ui/components/dialog";
import { Input } from "@expent/ui/components/input";
import { Label } from "@expent/ui/components/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@expent/ui/components/select";
import {
  ArrowDownLeftIcon,
  ArrowUpRightIcon,
  CheckCircle2Icon,
  ClockIcon,
  HistoryIcon,
  NotebookTabsIcon,
  PlusIcon,
  WalletIcon,
} from "lucide-react";
import * as React from "react";
import { useContacts } from "@/hooks/use-contacts";
import { useLedgerTabs } from "@/hooks/use-p2p";
import { useWallets } from "@/hooks/use-wallets";

export default function LedgerTabsPage() {
  const [isCreateDialogOpen, setIsCreateDialogOpen] = React.useState(false);
  const [title, setTitle] = React.useState("");
  const [description, setDescription] = React.useState("");
  const [targetAmount, setTargetAmount] = React.useState("");
  const [tabType, setTabType] = React.useState("LENT");
  const [contactId, setContactId] = React.useState("none");

  const { contacts } = useContacts();
  const { ledgerTabs, isLoading, createMutation } = useLedgerTabs();

  const handleCreate = () => {
    createMutation.mutate(
      {
        title,
        description: description || null,
        target_amount: parseFloat(targetAmount),
        tab_type: tabType as LedgerTab["tab_type"],
        counterparty_id: contactId !== "none" ? contactId : null,
      },
      {
        onSuccess: () => {
          setIsCreateDialogOpen(false);
          setTitle("");
          setDescription("");
          setTargetAmount("");
          setContactId("none");
        },
      },
    );
  };

  return (
    <div className="flex flex-1 flex-col gap-6 p-4 md:p-6 lg:p-8 max-w-7xl mx-auto w-full">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Ledger Tabs</h1>
          <p className="text-muted-foreground text-sm">Track money lent to or borrowed from others over time.</p>
        </div>
        <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
          <DialogTrigger render={<Button />}>
            <PlusIcon className="mr-2 h-4 w-4" /> New Tab
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Create Ledger Tab</DialogTitle>
              <DialogDescription>Start tracking a new shared balance with someone.</DialogDescription>
            </DialogHeader>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label htmlFor="title">Title</Label>
                <Input
                  id="title"
                  value={title}
                  onChange={(e) => setTitle(e.target.value)}
                  placeholder="e.g. Goa Trip Split, Rent"
                />
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div className="grid gap-2">
                  <Label htmlFor="type">Type</Label>
                  <Select value={tabType} onValueChange={(val) => setTabType(val || "LENT")}>
                    <SelectTrigger id="type">
                      <SelectValue />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="LENT">Money I Lent</SelectItem>
                      <SelectItem value="BORROWED">Money I Borrowed</SelectItem>
                    </SelectContent>
                  </Select>
                </div>
                <div className="grid gap-2">
                  <Label htmlFor="amount">Target Amount (₹)</Label>
                  <Input
                    id="amount"
                    type="number"
                    value={targetAmount}
                    onChange={(e) => setTargetAmount(e.target.value)}
                    placeholder="0.00"
                  />
                </div>
              </div>
              <div className="grid gap-2">
                <Label htmlFor="contact">Contact (Optional)</Label>
                <Select value={contactId} onValueChange={(val) => setContactId(val || "none")}>
                  <SelectTrigger id="contact">
                    <SelectValue placeholder="Select contact" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="none">No Contact linked</SelectItem>
                    {contacts?.map((c) => (
                      <SelectItem key={c.id} value={c.id}>
                        {c.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              <div className="grid gap-2">
                <Label htmlFor="desc">Description</Label>
                <Input
                  id="desc"
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  placeholder="Additional context..."
                />
              </div>
            </div>
            <DialogFooter>
              <Button variant="outline" onClick={() => setIsCreateDialogOpen(false)}>
                Cancel
              </Button>
              <Button onClick={handleCreate} disabled={!title || !targetAmount || createMutation.isPending}>
                Create Tab
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      {isLoading ? (
        <div className="space-y-4">
          {[1, 2].map((i) => (
            <Card key={i} className="h-32 animate-pulse bg-muted/50" />
          ))}
        </div>
      ) : !ledgerTabs || ledgerTabs.length === 0 ? (
        <Card className="border-dashed py-20">
          <CardContent className="flex flex-col items-center text-center">
            <div className="bg-muted p-4 rounded-full mb-4">
              <NotebookTabsIcon className="h-10 w-10 text-muted-foreground/40" />
            </div>
            <h3 className="text-lg font-medium">No ledger tabs yet</h3>
            <p className="text-sm text-muted-foreground mt-1 max-w-xs">
              Keep track of ongoing debts and repayments with friends or family.
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="grid gap-4 sm:grid-cols-2">
          {ledgerTabs.map((tab) => (
            <LedgerTabCard key={tab.id} tab={tab} />
          ))}
        </div>
      )}
    </div>
  );
}

function LedgerTabCard({ tab }: { tab: LedgerTab }) {
  const isLent = tab.tab_type === "LENT";
  const [isRepaymentOpen, setIsRepaymentOpen] = React.useState(false);

  const statusBadge = () => {
    switch (tab.status) {
      case "SETTLED":
        return (
          <Badge variant="outline" className="bg-green-50 text-green-700 border-green-200">
            <CheckCircle2Icon className="h-3 w-3 mr-1" /> Settled
          </Badge>
        );
      case "PARTIALLY_PAID":
        return (
          <Badge variant="outline" className="bg-blue-50 text-blue-700 border-blue-200">
            <ClockIcon className="h-3 w-3 mr-1" /> Partial
          </Badge>
        );
      default:
        return (
          <Badge variant="outline" className="bg-orange-50 text-orange-700 border-orange-200">
            Open
          </Badge>
        );
    }
  };

  return (
    <Card className="hover:border-primary/30 transition-colors shadow-sm overflow-hidden">
      <CardHeader className="p-4 flex flex-row items-start justify-between space-y-0 pb-2">
        <div className="space-y-1">
          <div className="flex items-center gap-2">
            <CardTitle className="text-base">{tab.title}</CardTitle>
            {statusBadge()}
          </div>
          <CardDescription className="text-xs">{tab.description || "No description"}</CardDescription>
        </div>
        <div className={`p-2 rounded-lg ${isLent ? "bg-rose-50 text-rose-600" : "bg-emerald-50 text-emerald-600"}`}>
          {isLent ? <ArrowUpRightIcon className="h-4 w-4" /> : <ArrowDownLeftIcon className="h-4 w-4" />}
        </div>
      </CardHeader>
      <CardContent className="px-4 py-2">
        <div className="flex justify-between items-end">
          <div>
            <p className="text-[10px] uppercase font-bold text-muted-foreground tracking-widest">Target Amount</p>
            <p className="text-xl font-bold font-mono">₹{parseFloat(tab.target_amount).toLocaleString()}</p>
          </div>
          <div className="text-right">
            <p className="text-[10px] text-muted-foreground italic">
              Created {new Date(tab.created_at).toLocaleDateString()}
            </p>
          </div>
        </div>
      </CardContent>
      <CardFooter className="px-4 py-3 bg-muted/20 border-t flex justify-between gap-2">
        <Button variant="ghost" size="sm" className="h-8 text-xs">
          <HistoryIcon className="h-3 w-3 mr-1" /> View History
        </Button>
        <RepaymentDialog tab={tab} open={isRepaymentOpen} onOpenChange={setIsRepaymentOpen} />
      </CardFooter>
    </Card>
  );
}

function RepaymentDialog({
  tab,
  open,
  onOpenChange,
}: {
  tab: LedgerTab;
  open: boolean;
  onOpenChange: (o: boolean) => void;
}) {
  const [amount, setAmount] = React.useState(tab.target_amount.toString());
  const [walletId, setWalletId] = React.useState("none");

  const { wallets } = useWallets();
  const { repaymentMutation } = useLedgerTabs();

  const handleRepay = () => {
    repaymentMutation.mutate(
      {
        id: tab.id,
        data: {
          amount: parseFloat(amount),
          wallet_id: walletId !== "none" ? walletId : null,
        },
      },
      {
        onSuccess: () => onOpenChange(false),
      },
    );
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogTrigger render={<Button size="sm" className="h-8 text-xs" variant="secondary" />}>
        Register Repayment
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Register Repayment</DialogTitle>
          <DialogDescription>Mark money as returned for "{tab.title}".</DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="grid gap-2">
            <Label htmlFor="repay-amount">Amount (₹)</Label>
            <Input id="repay-amount" type="number" value={amount} onChange={(e) => setAmount(e.target.value)} />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="repay-wallet">Source Wallet</Label>
            <div className="relative">
              <WalletIcon className="absolute left-2.5 top-2.5 h-4 w-4 text-muted-foreground" />
              <Select value={walletId} onValueChange={(val) => setWalletId(val || "none")}>
                <SelectTrigger className="pl-9">
                  <SelectValue placeholder="Select wallet" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="none">No Wallet (Cash)</SelectItem>
                  {wallets?.map((w) => (
                    <SelectItem key={w.id} value={w.id}>
                      {w.name} (₹{parseFloat(w.balance).toLocaleString()})
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleRepay} disabled={!amount || repaymentMutation.isPending}>
            Confirm Repayment
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
