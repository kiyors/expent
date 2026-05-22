"use client";

import type { ContactIdentifier, Transaction } from "@expent/types";
import { Badge } from "@expent/ui/components/badge";
import { Button } from "@expent/ui/components/button";
import { Card, CardContent, CardHeader, CardTitle } from "@expent/ui/components/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@expent/ui/components/dialog";
import { toast } from "@expent/ui/components/goey-toaster";
import { Input } from "@expent/ui/components/input";
import { Label } from "@expent/ui/components/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@expent/ui/components/select";
import { Separator } from "@expent/ui/components/separator";
import {
  ArrowLeftIcon,
  CheckIcon,
  CopyIcon,
  PhoneIcon,
  PlusIcon,
  ReceiptIcon,
  Trash2Icon,
  UserIcon,
  WalletIcon,
} from "lucide-react";
import { useParams, useRouter } from "next/navigation";
import * as React from "react";
import { DataTable } from "@/components/data-table/data-table";
import { useContactDetail, useContacts } from "@/hooks/use-contacts";
import type { Column } from "@/lib/data-table-types";

export default function ContactDetailPage() {
  const { id } = useParams<{ id: string }>();
  const router = useRouter();
  const [isAddIdDialogOpen, setIsAddIdDialogOpen] = React.useState(false);
  const [newIdType, setNewIdType] = React.useState("UPI");
  const [newIdValue, setNewIdValue] = React.useState("");

  const { contactData, isLoading, addIdentifierMutation } = useContactDetail(id);
  const { deleteMutation } = useContacts();
  const [_isPending, startTransition] = React.useTransition();
  if (isLoading) {
    return <div className="p-8 text-center">Loading contact details...</div>;
  }

  if (!contactData) {
    return <div className="p-8 text-center text-destructive">Contact not found</div>;
  }

  const { contact, identifiers, transactions } = contactData;

  const txnColumns = [
    {
      key: "date",
      label: "Date",
      format: { kind: "date", dateFormat: "short" },
    },
    { key: "purpose_tag", label: "Description" },
    {
      key: "direction",
      label: "Direction",
      format: { kind: "badge", colorMap: { IN: "success", OUT: "danger" } },
    },
    {
      key: "amount",
      label: "Amount",
      format: { kind: "currency", currency: "INR" },
      align: "right",
    },
  ] as Column<Transaction>[];

  return (
    <div className="flex flex-1 flex-col gap-6 p-4 md:p-6 lg:p-8 max-w-5xl mx-auto w-full">
      <div className="flex items-center gap-4">
        <Button variant="ghost" size="icon" onClick={() => router.back()} aria-label="Go back">
          <ArrowLeftIcon className="size-4" />
        </Button>
        <div className="flex-1">
          <h1 className="text-2xl font-semibold tracking-tight">{contact.name}</h1>
          <div className="flex items-center gap-2 text-sm text-muted-foreground mt-1">
            {contact.phone && (
              <span className="flex items-center gap-1">
                <PhoneIcon className="size-3" /> {contact.phone}
              </span>
            )}
            {contact.phone && identifiers.length > 0 && <span>•</span>}
            {identifiers.length > 0 && <span>{identifiers.length} identifier(s)</span>}
          </div>
        </div>
        <Button
          variant="outline"
          size="sm"
          className="text-destructive hover:bg-destructive/10"
          onClick={() => {
            if (confirm("Are you sure you want to remove this contact from your list?")) {
              deleteMutation.mutate(id, {
                onSuccess: () => {
                  startTransition(() => {
                    router.push("/contacts");
                  });
                },
              });
            }
          }}
        >
          <Trash2Icon className="size-4 mr-2" /> Remove
        </Button>
      </div>

      <div className="grid gap-6 md:grid-cols-3">
        <div className="md:col-span-1 gap-y-6">
          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-sm font-medium flex items-center justify-between">
                Identifiers
                <Dialog open={isAddIdDialogOpen} onOpenChange={setIsAddIdDialogOpen}>
                  <DialogTrigger render={<Button variant="ghost" size="icon-xs" aria-label="Add identifier" />}>
                    <PlusIcon className="h-3.5 w-3.5" />
                  </DialogTrigger>
                  <DialogContent>
                    <DialogHeader>
                      <DialogTitle>Add Identifier</DialogTitle>
                      <DialogDescription>Add a UPI ID, Phone, or Bank Account for this contact.</DialogDescription>
                    </DialogHeader>
                    <div className="grid gap-4 py-4">
                      <div className="grid gap-2">
                        <Label htmlFor="type">Type</Label>
                        <Select value={newIdType} onValueChange={(val) => setNewIdType(val || "UPI")}>
                          <SelectTrigger>
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="UPI">UPI ID</SelectItem>
                            <SelectItem value="PHONE">Phone Number</SelectItem>
                            <SelectItem value="BANK_ACC">Bank Account</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                      <div className="grid gap-2">
                        <Label htmlFor="value">Value</Label>
                        <Input
                          id="value"
                          value={newIdValue}
                          onChange={(e) => setNewIdValue(e.target.value)}
                          placeholder="e.g. name@upi"
                        />
                      </div>
                    </div>
                    <DialogFooter>
                      <Button variant="outline" onClick={() => setIsAddIdDialogOpen(false)}>
                        Cancel
                      </Button>
                      <Button
                        onClick={() =>
                          addIdentifierMutation.mutate(
                            { type: newIdType, value: newIdValue },
                            {
                              onSuccess: () => setIsAddIdDialogOpen(false),
                            },
                          )
                        }
                        disabled={!newIdValue || addIdentifierMutation.isPending}
                      >
                        Add Identifier
                      </Button>
                    </DialogFooter>
                  </DialogContent>
                </Dialog>
              </CardTitle>
            </CardHeader>
            <CardContent className="gap-y-3">
              {identifiers.length === 0 ? (
                <p className="text-xs text-muted-foreground italic">No identifiers added yet.</p>
              ) : (
                identifiers.map((id: ContactIdentifier) => <IdentifierChip key={id.id} identifier={id} />)
              )}
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="pb-3">
              <CardTitle className="text-sm font-medium">Insights</CardTitle>
            </CardHeader>
            <CardContent className="gap-y-4">
              <div className="flex justify-between items-center">
                <span className="text-xs text-muted-foreground">Total Transactions</span>
                <span className="font-medium">{transactions.length}</span>
              </div>
              <div className="flex justify-between items-center">
                <span className="text-xs text-muted-foreground">Total Volume</span>
                <span className="font-bold text-primary">
                  ₹
                  {transactions.reduce((acc: number, t: Transaction) => acc + parseFloat(t.amount), 0).toLocaleString()}
                </span>
              </div>
              <Separator />
              <div className="flex items-center gap-2">
                <Badge variant="outline" className="bg-primary/5 text-[10px]">
                  <UserIcon className="size-3 mr-1" /> Contact Detail
                </Badge>
              </div>
            </CardContent>
          </Card>
        </div>

        <div className="md:col-span-2">
          <Card className="h-full flex flex-col">
            <CardHeader className="border-b bg-muted/10">
              <CardTitle className="text-base flex items-center gap-2">
                <ReceiptIcon className="size-4 text-primary" /> Transaction History
              </CardTitle>
            </CardHeader>
            <CardContent className="p-0 flex-1">
              <DataTable
                id="contact-transactions"
                columns={txnColumns}
                data={transactions}
                rowIdKey="id"
                emptyMessage="No transactions with this contact yet."
                locale="en-IN"
              />
            </CardContent>
          </Card>
        </div>
      </div>
    </div>
  );
}

function IdentifierChip({ identifier }: { identifier: ContactIdentifier }) {
  const [copied, setCopied] = React.useState(false);

  const copy = () => {
    navigator.clipboard.writeText(identifier.value);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
    toast.success("Copied to clipboard");
  };

  return (
    <div className="group flex items-center justify-between p-2 rounded-lg border bg-muted/30 text-xs transition-colors hover:bg-muted/50">
      <div className="flex items-center gap-2 min-w-0">
        <div className="size-6 rounded-md bg-background flex items-center justify-center border shadow-xs">
          {identifier.type === "UPI" ? (
            <WalletIcon className="size-3 text-primary" />
          ) : (
            <PhoneIcon className="size-3" />
          )}
        </div>
        <div className="min-w-0">
          <p className="font-mono truncate">{identifier.value}</p>
          <p className="text-[10px] text-muted-foreground uppercase">{identifier.type}</p>
        </div>
      </div>
      <Button
        variant="ghost"
        size="icon-xs"
        onClick={copy}
        className="opacity-0 group-hover:opacity-100"
        aria-label="Copy identifier"
      >
        {copied ? <CheckIcon className="size-3 text-green-600" /> : <CopyIcon className="size-3" />}
      </Button>
    </div>
  );
}
