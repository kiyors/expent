import type { Contact, ContactIdentifier, Transaction } from "@expent/types";
import { toast } from "@expent/ui/components/goey-toaster";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api-client";
import { useSession } from "@/lib/auth-client";

export function useContacts() {
  const session = useSession();
  const queryClient = useQueryClient();

  const query = useQuery({
    queryKey: ["contacts"],
    queryFn: () => api.get<Contact[]>("/api/contacts"),
    enabled: !!session.data,
    staleTime: 1000 * 60 * 5, // 5 minutes
  });

  const createMutation = useMutation({
    mutationFn: (data: { name: string; phone?: string | null }) => api.post<Contact>("/api/contacts", data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["contacts"] });
      toast.success("Contact added");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, data }: { id: string; data: Partial<Contact> }) => api.put<Contact>(`/api/contacts/${id}`, data),
    onMutate: async ({ id, data }) => {
      await queryClient.cancelQueries({ queryKey: ["contacts"] });
      const previousContacts = queryClient.getQueryData<Contact[]>(["contacts"]);

      queryClient.setQueryData<Contact[]>(["contacts"], (old) => {
        if (!old) return old;
        return old.map((c) => (c.id === id ? { ...c, ...data } : c));
      });

      return { previousContacts };
    },
    onError: (err, _variables, context) => {
      queryClient.setQueryData(["contacts"], context?.previousContacts);
      toast.error(err.message);
    },
    onSettled: () => {
      queryClient.invalidateQueries({ queryKey: ["contacts"] });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => api.delete(`/api/contacts/${id}`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["contacts"] });
      toast.success("Contact removed");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  return {
    contacts: query.data,
    isLoading: query.isLoading,
    error: query.error,
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
    mutationFn: (data: { primary_id: string; secondary_id: string }) => api.post<Contact>("/api/contacts/merge", data),
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
    queryFn: () =>
      api.get<{
        contact: Contact;
        identifiers: ContactIdentifier[];
        transactions: Transaction[];
      }>(`/api/contacts/${id}`),
    enabled: !!id,
    staleTime: 1000 * 60 * 5, // 5 minutes
  });

  const addIdentifierMutation = useMutation({
    mutationFn: (data: { type: string; value: string }) =>
      api.post<ContactIdentifier>(`/api/contacts/${id}/identifiers`, data),
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
