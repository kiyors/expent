import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";

import { CommandCenter } from "@/components/layout/CommandCenter";
import { GlobalModals } from "@/components/layout/GlobalModals";
import { HotkeyHelp } from "@/components/layout/HotkeyHelp";
import { SidebarWrapper } from "@/components/layout/SidebarWrapper";
import { getSession } from "@/lib/Auth.functions";

export const Route = createFileRoute("/_dashboard")({
  beforeLoad: async ({ location }) => {
    const session = await getSession();

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
      <CommandCenter />
      <HotkeyHelp />
      <GlobalModals />
    </SidebarWrapper>
  );
}
