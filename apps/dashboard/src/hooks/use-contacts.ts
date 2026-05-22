import type {
  AddIdentifierRequest,
  Contact,
  ContactDetail,
  ContactIdentifier,
  CreateContactRequest,
  MergeContactsRequest,
  Transaction,
  UpdateContactRequest,
  ValidationResult,
} from "@expent/types";
import { toast } from "@expent/ui/components/goey-toaster";
import { useLiveQuery } from "@tanstack/react-db";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api-client";
import { useSession } from "@/lib/auth-client";
import { db } from "@/lib/db";
import { validateContactWasm } from "@expent/wasm";

export function useContacts() {
  const session = useSession();
  const queryClient = useQueryClient();

  const query = useLiveQuery((q) => q.from({ contacts: db.contacts }), [session.data]);

  const createMutation = useMutation({
    mutationFn: async (data: CreateContactRequest) => {
      // 0. Shared WASM Validation
      const result = (await validateContactWasm(data.name)) as unknown as ValidationResult;
      if (!result.is_valid) {
        throw new Error(result.errors.map((e) => `${e.field}: ${e.message}`).join(", "));
      }
      return api.post<Contact, CreateContactRequest>("/api/contacts", data);
    },
    onSuccess: (newContact) => {
      db.contacts.insert(newContact);
      toast.success("Contact added");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  const updateMutation = useMutation({
    mutationFn: async ({ id, data }: { id: string; data: UpdateContactRequest }) => {
      // 0. Shared WASM Validation
      if (data.name) {
        const result = (await validateContactWasm(data.name)) as unknown as ValidationResult;
        if (!result.is_valid) {
          throw new Error(result.errors.map((e) => `${e.field}: ${e.message}`).join(", "));
        }
      }
      return api.put<Contact, UpdateContactRequest>(`/api/contacts/${id}`, data);
    },
    onMutate: async ({ id, data }) => {
      const previousContact = db.contacts.get(id);

      db.contacts.update(id, (draft) => {
        Object.assign(draft, data);
      });

      return { previousContact };
    },
    onError: (err, { id }, context) => {
      if (context?.previousContact) {
        db.contacts.update(id, (draft) => {
          Object.assign(draft, context.previousContact);
        });
      }
      toast.error(err.message);
    },
    onSuccess: (updatedContact, { id }) => {
      db.contacts.update(id, (draft) => {
        Object.assign(draft, updatedContact);
      });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => api.delete(`/api/contacts/${id}`),
    onMutate: async (id) => {
      const previousContact = db.contacts.get(id);
      db.contacts.delete(id);
      return { previousContact };
    },
    onError: (err, id, context) => {
      if (context?.previousContact) {
        db.contacts.insert(context.previousContact);
      }
      toast.error(err.message);
    },
    onSuccess: () => {
      toast.success("Contact removed");
    },
  });

  return {
    contacts: query.data as unknown as Contact[],
    isLoading: query.isLoading,
    error: query.isError ? "Error loading contacts" : null,
    createMutation,
    updateMutation,
    deleteMutation,
  };
}

export function useMergeContacts() {
  const session = useSession();
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ["contacts-suggestions"],
    queryFn: () => api.get<{ contacts: Contact[]; reason: string }[]>("/api/contacts/suggestions"),
    enabled: !!session.data,
  });

  const mergeMutation = useMutation({
    mutationFn: (data: MergeContactsRequest) => api.post<Contact, MergeContactsRequest>("/api/contacts/merge", data),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: ["contacts"] });
      queryClient.invalidateQueries({ queryKey: ["contacts-suggestions"] });
      queryClient.invalidateQueries({ queryKey: ["contact-detail", variables.primary_id] });
      toast.success("Contacts merged successfully");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  return {
    suggestions: query.data,
    isLoading: query.isLoading,
    error: query.error,
    mergeMutation,
  };
}

export function useContactDetail(id: string) {
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ["contact-detail", id],
    queryFn: () => api.get<ContactDetail>(`/api/contacts/${id}`),
    enabled: !!id,
    staleTime: 1000 * 60 * 5, // 5 minutes
  });

  const addIdentifierMutation = useMutation({
    mutationFn: (data: AddIdentifierRequest) =>
      api.post<ContactIdentifier, AddIdentifierRequest>(`/api/contacts/${id}/identifiers`, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["contact-detail", id] });
      toast.success("Identifier added");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  return {
    contactData: query.data,
    isLoading: query.isLoading,
    addIdentifierMutation,
  };
}
