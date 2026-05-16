"use client";

import type { Contact } from "@expent/types";
import { Alert, AlertDescription, AlertTitle } from "@expent/ui/components/alert";
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
import { Input } from "@expent/ui/components/input";
import { Label } from "@expent/ui/components/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@expent/ui/components/select";
import type { UseMutationResult } from "@tanstack/react-query";
import {
  AlertCircleIcon,
  ChevronRightIcon,
  GitMergeIcon,
  PhoneIcon,
  PinIcon,
  PlusIcon,
  SearchIcon,
  UserIcon,
} from "lucide-react";
import { motion } from "motion/react";
import { useRouter } from "next/navigation";
import * as React from "react";
import { useContacts, useMergeContacts } from "@/hooks/use-contacts";

export default function ContactsPage() {
  const router = useRouter();
  const [_isPending, startTransition] = React.useTransition();
  const [searchQuery, setSearchSearchQuery] = React.useState("");
  const [isAddDialogOpen, setIsAddDialogOpen] = React.useState(false);
  const [newName, setNewName] = React.useState("");
  const [newPhone, setNewPhone] = React.useState("");

  const { contacts, isLoading, createMutation, updateMutation } = useContacts();
  const { suggestions, mergeMutation } = useMergeContacts();

  const handleContactClick = (id: string) => {
    startTransition(() => {
      router.push(`/contacts/${id}`);
    });
  };

  const handleCreate = () => {
    createMutation.mutate(
      { name: newName, phone: newPhone || null },
      {
        onSuccess: () => {
          setIsAddDialogOpen(false);
          setNewName("");
          setNewPhone("");
        },
      },
    );
  };

  const filteredContacts = React.useMemo(() => {
    if (!contacts) return [];
    return contacts.filter((c) => c.name.toLowerCase().includes(searchQuery.toLowerCase()));
  }, [contacts, searchQuery]);

  const pinnedContacts = filteredContacts.filter((c) => c.is_pinned);
  const otherContacts = filteredContacts.filter((c) => !c.is_pinned);

  return (
    <div className="flex flex-1 flex-col gap-4 sm:gap-6 p-4 md:p-6 lg:p-8 max-w-7xl mx-auto w-full">
      <div className="flex flex-wrap items-end justify-between gap-2 mb-2">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Contact List</h2>
          <p className="text-muted-foreground">Manage your frequent counterparties, friends, and vendors.</p>
        </div>
        <Dialog open={isAddDialogOpen} onOpenChange={setIsAddDialogOpen}>
          <DialogTrigger render={<Button />}>
            <PlusIcon className="mr-2 h-4 w-4" /> Add Contact
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Add New Contact</DialogTitle>
              <DialogDescription>Create a new contact to track transactions with them.</DialogDescription>
            </DialogHeader>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label htmlFor="name">Full Name</Label>
                <Input
                  id="name"
                  value={newName}
                  onChange={(e) => setNewName(e.target.value)}
                  placeholder="e.g. John Doe"
                />
              </div>
              <div className="grid gap-2">
                <Label htmlFor="phone">Phone Number (Optional)</Label>
                <Input
                  id="phone"
                  name="phone"
                  value={newPhone}
                  onChange={(e) => setNewPhone(e.target.value)}
                  placeholder="+91…"
                  autoComplete="tel"
                />
              </div>
            </div>
            <DialogFooter>
              <Button variant="outline" onClick={() => setIsAddDialogOpen(false)}>
                Cancel
              </Button>
              <Button onClick={handleCreate} disabled={!newName || createMutation.isPending}>
                {createMutation.isPending ? "Adding…" : "Add Contact"}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      <div className="relative">
        <SearchIcon className="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
        <Input
          placeholder="Search contacts by name…"
          className="pl-10 h-11 bg-card shadow-xs"
          value={searchQuery}
          onChange={(e) => setSearchSearchQuery(e.target.value)}
        />
      </div>

      {suggestions && suggestions.length > 0 && (
        <MergeSuggestionsBanner suggestions={suggestions} mergeMutation={mergeMutation} />
      )}

      {isLoading ? (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {[1, 2, 3].map((i) => (
            <Card key={i} className="h-32 animate-pulse bg-muted/50" />
          ))}
        </div>
      ) : filteredContacts.length === 0 ? (
        <Card className="border-dashed py-20">
          <CardContent className="flex flex-col items-center text-center">
            <div className="bg-muted p-4 rounded-full mb-4">
              <UserIcon className="h-10 w-10 text-muted-foreground/40" />
            </div>
            <h3 className="text-lg font-medium">No contacts found</h3>
            <p className="text-sm text-muted-foreground mt-1 max-w-xs">
              {searchQuery
                ? `No results for "${searchQuery}"`
                : "Start by adding your first contact to track splits and payments."}
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-8">
          {pinnedContacts.length > 0 && (
            <section className="space-y-4">
              <h2 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground flex items-center gap-2">
                <PinIcon className="h-3 w-3 rotate-45" /> Pinned
              </h2>
              <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
                {pinnedContacts.map((contact) => (
                  <ContactCard
                    key={contact.id}
                    contact={contact}
                    onPin={() =>
                      updateMutation.mutate({
                        id: contact.id,
                        data: { is_pinned: false },
                      })
                    }
                    onClick={() => handleContactClick(contact.id)}
                  />
                ))}
              </div>
            </section>
          )}

          <section className="space-y-4">
            <h2 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
              {pinnedContacts.length > 0 ? "All Contacts" : "All Contacts"}
            </h2>
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
              {otherContacts.map((contact) => (
                <ContactCard
                  key={contact.id}
                  contact={contact}
                  onPin={() =>
                    updateMutation.mutate({
                      id: contact.id,
                      data: { is_pinned: true },
                    })
                  }
                  onClick={() => handleContactClick(contact.id)}
                />
              ))}
            </div>
          </section>
        </div>
      )}
    </div>
  );
}

function ContactCard({ contact, onPin, onClick }: { contact: Contact; onPin: () => void; onClick: () => void }) {
  return (
    <motion.div whileHover={{ y: -2 }} whileTap={{ scale: 0.98 }} className="h-full">
      <Card
        className="group hover:border-primary/50 transition-all cursor-pointer relative overflow-hidden h-full"
        onClick={onClick}
      >
        <CardHeader className="p-4 flex flex-row items-center gap-4 space-y-0">
          <div className="size-12 rounded-full bg-primary/10 flex items-center justify-center text-primary font-bold text-lg">
            {contact.name.charAt(0).toUpperCase()}
          </div>
          <div className="flex-1 min-w-0">
            <CardTitle className="text-base truncate">{contact.name}</CardTitle>
            <div className="flex items-center text-xs text-muted-foreground gap-2 mt-0.5">
              {contact.phone && (
                <span className="flex items-center gap-1">
                  <PhoneIcon className="h-3 w-3" /> {contact.phone}
                </span>
              )}
            </div>
          </div>
          <div className="flex flex-col items-end gap-2">
            <Button
              variant="ghost"
              size="icon-xs"
              className={`h-7 w-7 rounded-full opacity-0 group-hover:opacity-100 transition-opacity ${contact.is_pinned ? "opacity-100 text-primary" : ""}`}
              onClick={(e) => {
                e.stopPropagation();
                onPin();
              }}
              aria-label={contact.is_pinned ? "Unpin contact" : "Pin contact"}
            >
              <PinIcon className={`h-3.5 w-3.5 ${contact.is_pinned ? "fill-current rotate-45" : ""}`} />
            </Button>
            <ChevronRightIcon className="h-4 w-4 text-muted-foreground/30 group-hover:text-primary transition-colors" />
          </div>
        </CardHeader>
      </Card>
    </motion.div>
  );
}

function MergeSuggestionsBanner({
  suggestions,
  mergeMutation,
}: {
  suggestions: { contacts: Contact[]; reason: string }[];
  mergeMutation: UseMutationResult<Contact, Error, { primary_id: string; secondary_id: string }, unknown>;
}) {
  const [isDialogOpen, setIsDialogOpen] = React.useState(false);
  const [selectedSuggestion, setSelectedSuggestion] = React.useState<{
    contacts: Contact[];
    reason: string;
  } | null>(null);
  const [primaryId, setPrimaryId] = React.useState<string | null>(null);
  const [showAll, setShowAll] = React.useState(false);

  const handleOpenMerge = (suggestion: { contacts: Contact[]; reason: string }) => {
    setSelectedSuggestion(suggestion);
    setPrimaryId(suggestion.contacts[0].id);
    setIsDialogOpen(true);
  };

  const handleMerge = () => {
    if (!selectedSuggestion || !primaryId) return;

    const secondaryId = selectedSuggestion.contacts.find((c: Contact) => c.id !== primaryId)?.id;
    if (!secondaryId) return;

    mergeMutation.mutate(
      { primary_id: primaryId, secondary_id: secondaryId },
      {
        onSuccess: () => {
          setIsDialogOpen(false);
          setSelectedSuggestion(null);
        },
      },
    );
  };

  return (
    <Alert className="bg-primary/5 border-primary/20">
      <AlertCircleIcon className="h-4 w-4 text-primary" />
      <AlertTitle className="text-primary font-medium">Merge Contacts ({suggestions.length})</AlertTitle>
      <AlertDescription className="mt-2 flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <span className="text-sm">
          We found duplicate or similar contacts. Merging them keeps your transactions clean.
        </span>
        <div className="flex gap-2 flex-wrap">
          {suggestions.slice(0, showAll ? suggestions.length : 2).map((s, _idx) => (
            <Button
              key={`${s.contacts[0].id}-${s.contacts[1].id}`}
              variant="outline"
              size="sm"
              onClick={() => handleOpenMerge(s)}
              className="gap-2 bg-background"
            >
              <GitMergeIcon className="h-3 w-3" />
              Merge "{s.contacts[0].name}"
            </Button>
          ))}
          {suggestions.length > 2 && !showAll && (
            <Button variant="ghost" size="sm" onClick={() => setShowAll(true)} className="text-primary text-xs">
              View {suggestions.length - 2} more...
            </Button>
          )}
        </div>
      </AlertDescription>

      <Dialog open={isDialogOpen} onOpenChange={setIsDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Merge Contacts</DialogTitle>
            <DialogDescription>
              Reason: {selectedSuggestion?.reason}. Choose which contact to keep as the primary. The other will be
              merged into it and deleted. All transactions and identifiers will be preserved.
            </DialogDescription>
          </DialogHeader>

          {selectedSuggestion && (
            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label>Primary Contact to Keep</Label>
                {(() => {
                  const primaryContact = selectedSuggestion.contacts.find((c: Contact) => c.id === primaryId);
                  return (
                    <Select value={primaryId} onValueChange={setPrimaryId}>
                      <SelectTrigger>
                        <SelectValue placeholder="Select primary contact">
                          {primaryContact?.name} {primaryContact?.phone ? `(${primaryContact.phone})` : ""}
                        </SelectValue>
                      </SelectTrigger>
                      <SelectContent>
                        {selectedSuggestion.contacts.map((c: Contact) => (
                          <SelectItem key={c.id} value={c.id}>
                            {c.name} {c.phone ? `(${c.phone})` : ""}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  );
                })()}
              </div>
            </div>
          )}

          <DialogFooter>
            <Button variant="outline" onClick={() => setIsDialogOpen(false)}>
              Cancel
            </Button>
            <Button onClick={handleMerge} disabled={!primaryId || mergeMutation.isPending}>
              {mergeMutation.isPending ? "Merging..." : "Accept & Merge"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </Alert>
  );
}
