"use client";

import type { Category } from "@expent/types/db/Category";
import { Button } from "@expent/ui/components/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@expent/ui/components/card";
import { toast } from "@expent/ui/components/goey-toaster";
import { Input } from "@expent/ui/components/input";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { PlusIcon, TagIcon, Trash2Icon } from "lucide-react";
import * as React from "react";
import { api } from "@/lib/api-client";

export function CategoriesPanel() {
  const queryClient = useQueryClient();
  const [newName, setNewName] = React.useState("");

  const { data: categories, isLoading } = useQuery({
    queryKey: ["categories"],
    queryFn: () => api.get<Category[]>("/api/categories"),
  });

  const createMutation = useMutation({
    mutationFn: (name: string) => api.post("/api/categories", { name }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["categories"] });
      setNewName("");
      toast.success("Category added");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => api.delete(`/api/categories/${id}`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["categories"] });
      toast.success("Category deleted");
    },
    onError: (error: Error) => toast.error(error.message),
  });

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center gap-3">
          <div className="flex size-10 items-center justify-center rounded-full bg-primary/10 text-primary">
            <TagIcon className="size-5" />
          </div>
          <div>
            <CardTitle>Categories</CardTitle>
            <CardDescription>Manage your custom transaction tags</CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent className="gap-y-4">
        <div className="flex gap-2">
          <Input
            placeholder="New category name..."
            value={newName}
            onChange={(e) => setNewName(e.target.value)}
            onKeyDown={(e) => e.key === "Enter" && createMutation.mutate(newName)}
          />
          <Button onClick={() => createMutation.mutate(newName)} disabled={!newName || createMutation.isPending}>
            <PlusIcon className="size-4 mr-2" /> Add
          </Button>
        </div>

        <div className="flex flex-wrap gap-2 pt-2">
          {isLoading ? (
            <p className="text-sm text-muted-foreground">Loading categories...</p>
          ) : !categories || categories.length === 0 ? (
            <p className="text-sm text-muted-foreground italic">No custom categories yet.</p>
          ) : (
            categories.map((cat) => (
              <div
                key={cat.id}
                className="flex items-center gap-2 px-3 py-1 bg-muted rounded-full border text-sm group"
              >
                <span className="font-medium">{cat.name}</span>
                <button
                  type="button"
                  aria-label={`Delete ${cat.name} category`}
                  onClick={() => {
                    if (confirm(`Delete category "${cat.name}"?`)) {
                      deleteMutation.mutate(cat.id);
                    }
                  }}
                  className="text-muted-foreground hover:text-destructive transition-colors opacity-0 group-hover:opacity-100 focus-visible:opacity-100 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring rounded-sm"
                >
                  <Trash2Icon className="size-3" aria-hidden="true" />
                </button>
              </div>
            ))
          )}
        </div>
      </CardContent>
    </Card>
  );
}
