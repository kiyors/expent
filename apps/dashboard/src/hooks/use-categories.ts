import type { Category, CreateCategoryRequest } from "@expent/types";
import { useLiveQuery } from "@tanstack/react-db";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api-client";
import { useSession } from "@/lib/auth-client";
import { db } from "@/lib/db";

export function useCategories() {
  const _queryClient = useQueryClient();
  const session = useSession();

  const query = useLiveQuery((q) => q.from({ categories: db.categories }), [session.data]);

  const createMutation = useMutation({
    mutationFn: (data: CreateCategoryRequest) => api.post<Category, CreateCategoryRequest>("/api/categories", data),
    onSuccess: (newCat) => {
      db.categories.insert(newCat);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => api.delete(`/api/categories/${id}`),
    onMutate: async (id) => {
      const previousCat = db.categories.get(id);
      db.categories.delete(id);
      return { previousCat };
    },
    onError: (_err, _id, context) => {
      if (context?.previousCat) {
        db.categories.insert(context.previousCat);
      }
    },
  });

  return {
    categories: query.data as unknown as Category[],
    isLoading: query.isLoading,
    isError: query.isError,
    createMutation,
    deleteMutation,
  };
}
