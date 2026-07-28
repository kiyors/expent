import { CommandCenter } from "@/components/layout/command-center";
import { GlobalModals } from "@/components/layout/global-modals";
import { HotkeyHelp } from "@/components/layout/hotkey-help";
import { SidebarWrapper } from "@/components/layout/sidebar-wrapper";
import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";
import { getSession } from "@/lib/auth.functions";

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
