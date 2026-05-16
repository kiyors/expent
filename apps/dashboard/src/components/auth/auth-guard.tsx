"use client";

import { useRouter } from "next/navigation";
import { useEffect, useTransition } from "react";
import { useSession } from "@/lib/auth-client";

export function AuthGuard({ children }: { children: React.ReactNode }) {
  const session = useSession();
  const router = useRouter();
  const [_isPending, startTransition] = useTransition();

  useEffect(() => {
    if (!session.isPending && !session.data) {
      startTransition(() => {
        router.push("/sign-in");
      });
    }
  }, [session.data, session.isPending, router]);

  if (session.isPending || !session.data) {
    return <div className="flex h-screen items-center justify-center">Loading...</div>;
  }

  return <>{children}</>;
}
