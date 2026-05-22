"use client";

import type { Contact } from "@expent/types";
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
import * as React from "react";
import { useContacts } from "@/hooks/use-contacts";

interface CreateContactDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onCreated?: (contactId: string) => void;
}

export function CreateContactDialog({ open, onOpenChange, onCreated }: CreateContactDialogProps) {
  const [name, setName] = React.useState("");
  const [phone, setPhone] = React.useState("");

  const { createMutation } = useContacts();

  React.useEffect(() => {
    if (open) {
      setName("");
      setPhone("");
    }
  }, [open]);

  const handleSubmit = () => {
    if (!name.trim()) {
      toast.error("Contact name is required");
      return;
    }

    createMutation.mutate(
      {
        name: name.trim(),
        phone: phone.trim() || null,
      },
      {
        onSuccess: (data: Contact) => {
          toast.success("Contact created!");
          onOpenChange(false);
          if (onCreated && data?.id) {
            onCreated(data.id);
          }
        },
        onError: (err) => {
          toast.error(err.message || "Failed to create contact");
        },
      },
    );
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[420px]">
        <DialogHeader>
          <DialogTitle>Add New Contact</DialogTitle>
          <DialogDescription>Create a new contact to track transactions with them.</DialogDescription>
        </DialogHeader>

        <div className="grid gap-4 py-4">
          <div className="grid gap-2">
            <Label htmlFor="contact-name">Full Name</Label>
            <Input
              id="contact-name"
              placeholder="e.g. John Doe"
              value={name}
              onChange={(e) => setName(e.target.value)}
              autoComplete="off"
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="contact-phone">Phone Number (Optional)</Label>
            <Input
              id="contact-phone"
              name="contact-phone"
              placeholder="+91..."
              value={phone}
              onChange={(e) => setPhone(e.target.value)}
              autoComplete="tel"
            />
          </div>
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={handleSubmit} disabled={!name.trim() || createMutation.isPending}>
            {createMutation.isPending ? "Adding..." : "Add Contact"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
