import { queryOptions } from "@tanstack/react-query";
import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";

import { ClientOnly } from "@/components/ClientOnly";
import { CommandCenter } from "@/components/layout/CommandCenter";
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
  component: DashboardLayout,
});

function DashboardLayout() {
  return (
    <SidebarWrapper>
      <Outlet />
      <ClientOnly>
        <CommandCenter />
        <GlobalModals />
      </ClientOnly>
      <HotkeyHelp />
    </SidebarWrapper>
  );
}
