"use client";

import { Badge } from "@expent/ui/components/badge";
import { Button } from "@expent/ui/components/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@expent/ui/components/card";
import { toast } from "@expent/ui/components/goey-toaster";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { FingerprintIcon, KeyIcon, LaptopIcon, PlusIcon, SmartphoneIcon, Trash2Icon } from "lucide-react";
import { api } from "@/lib/api-client";
import { useSession } from "@/lib/auth-client";

interface Passkey {
  id: string;
  name: string | null;
  createdAt: string;
}

export function SecurityPanel() {
  const queryClient = useQueryClient();
  const session = useSession();

  const { data: passkeys } = useQuery({
    queryKey: ["passkeys"],
    queryFn: () => api.get<Passkey[]>("/api/auth/passkey/list"),
    enabled: !!session.data,
  });

  const { data: activeSessions } = useQuery({
    queryKey: ["active-sessions"],
    queryFn: () => api.get<any[]>("/api/auth/sessions"),
    enabled: !!session.data,
  });

  const revokeSessionMutation = useMutation({
    mutationFn: (id: string) => api.delete(`/api/auth/sessions/${id}`),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["active-sessions"] });
      toast.success("Session revoked");
    },
  });

  const addPasskeyMutation = useMutation({
    mutationFn: async () => {
      // In a real app, you'd use authClient.passkey.addPasskey()
      // For now we simulate or use the client if available
      toast.info("Passkey registration started...");
      // This is a placeholder for better-auth passkey client logic
    },
  });

  return (
    <div className="space-y-6">
      {/* Passkeys */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
              <FingerprintIcon className="h-5 w-5" />
            </div>
            <div>
              <CardTitle>Passkeys</CardTitle>
              <CardDescription>Passwordless authentication for your account</CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          {!passkeys || passkeys.length === 0 ? (
            <div className="text-center py-6 border rounded-lg bg-muted/10 border-dashed">
              <p className="text-sm text-muted-foreground">No passkeys registered.</p>
              <Button variant="link" size="sm" className="mt-1" onClick={() => addPasskeyMutation.mutate()}>
                Add your first passkey
              </Button>
            </div>
          ) : (
            <div className="space-y-3">
              {passkeys.map((pk) => (
                <div key={pk.id} className="flex items-center justify-between p-3 rounded-lg border bg-muted/30">
                  <div className="flex items-center gap-3">
                    <KeyIcon className="h-4 w-4 text-primary" />
                    <div>
                      <p className="text-sm font-medium">{pk.name || "Unnamed Passkey"}</p>
                      <p className="text-[10px] text-muted-foreground uppercase">
                        Added {new Date(pk.createdAt).toLocaleDateString()}
                      </p>
                    </div>
                  </div>
                  <Button variant="ghost" size="icon-xs" className="text-destructive">
                    <Trash2Icon className="h-3.5 w-3.5" />
                  </Button>
                </div>
              ))}
              <Button variant="outline" size="sm" className="w-full" onClick={() => addPasskeyMutation.mutate()}>
                <PlusIcon className="h-3 w-3 mr-2" /> Register New Passkey
              </Button>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Active Sessions */}
      <Card>
        <CardHeader>
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10 text-primary">
              <LaptopIcon className="h-5 w-5" />
            </div>
            <div>
              <CardTitle>Active Sessions</CardTitle>
              <CardDescription>Devices currently logged into your account</CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-3">
            {!activeSessions ? (
              <p className="text-sm text-muted-foreground italic">Loading sessions...</p>
            ) : (
              activeSessions.map((s) => (
                <div key={s.id} className="flex items-center justify-between p-3 rounded-lg border bg-card">
                  <div className="flex items-center gap-3">
                    {s.userAgent?.includes("Mobile") ? (
                      <SmartphoneIcon className="h-5 w-5 text-muted-foreground" />
                    ) : (
                      <LaptopIcon className="h-5 w-5 text-muted-foreground" />
                    )}
                    <div>
                      <div className="flex items-center gap-2">
                        <p className="text-sm font-medium">{s.userAgent || "Unknown Device"}</p>
                        {s.id === session.data?.session?.id && (
                          <Badge variant="secondary" className="text-[10px] h-4">
                            Current
                          </Badge>
                        )}
                      </div>
                      <p className="text-[10px] text-muted-foreground">
                        IP: {s.ipAddress || "Unknown"} • Last active {new Date(s.updatedAt).toLocaleString()}
                      </p>
                    </div>
                  </div>
                  {s.id !== session.data?.session?.id && (
                    <Button
                      variant="ghost"
                      size="sm"
                      className="text-destructive h-8"
                      onClick={() => revokeSessionMutation.mutate(s.id)}
                    >
                      Revoke
                    </Button>
                  )}
                </div>
              ))
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
