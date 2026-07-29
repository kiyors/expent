import { queryOptions } from "@tanstack/react-query";
import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";
import { Loader2Icon, AlertCircleIcon } from "lucide-react";

import { ClientOnly } from "@/components/ClientOnly";
import { CommandCenter } from "@/components/layout/CommandCenter";
import { DemoBanner } from "@/components/layout/DemoBanner";
import { GlobalModals } from "@/components/layout/GlobalModals";
import { HotkeyHelp } from "@/components/layout/HotkeyHelp";
import { SidebarWrapper } from "@/components/layout/SidebarWrapper";
import { getSession } from "@/lib/Auth.functions";

const sessionQueryOptions = () =>
  queryOptions({
    queryKey: ["auth-session"],
    queryFn: () => getSession(),
    staleTime: 5 * 60 * 1000,
  });

export const Route = createFileRoute("/_dashboard")({
  beforeLoad: async ({ location, context }) => {
    const session = await context.queryClient.ensureQueryData(sessionQueryOptions());

    if (!session) {
      throw redirect({
        to: "/sign-in",
        search: { redirect: location.href },
      });
    }

    return { user: session.user };
  },
  pendingComponent: () => (
    <div className="flex h-screen w-full items-center justify-center bg-background">
      <Loader2Icon className="h-8 w-8 animate-spin text-muted-foreground" />
    </div>
  ),
  errorComponent: ({ error }) => (
    <div className="flex h-screen w-full flex-col items-center justify-center bg-background text-destructive p-4">
      <AlertCircleIcon className="h-10 w-10 mb-4" />
      <h2 className="text-xl font-bold mb-2">Something went wrong</h2>
      <p className="text-sm text-center max-w-md">{error.message || "An unexpected error occurred."}</p>
    </div>
  ),
  component: DashboardLayout,
});

function DashboardLayout() {
  return (
    <SidebarWrapper>
      <DemoBanner />
      <Outlet />
      <ClientOnly>
        <CommandCenter />
        <GlobalModals />
      </ClientOnly>
      <HotkeyHelp />
    </SidebarWrapper>
  );
}
