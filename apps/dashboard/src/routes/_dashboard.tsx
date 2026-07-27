import { AuthGuard } from "@/components/auth/auth-guard";
import { CommandCenter } from "@/components/layout/command-center";
import { GlobalModals } from "@/components/layout/global-modals";
import { HotkeyHelp } from "@/components/layout/hotkey-help";
import { SidebarWrapper } from "@/components/layout/sidebar-wrapper";
import { createFileRoute, Outlet } from "@tanstack/react-router";

export const Route = createFileRoute("/_dashboard")({
  component: DashboardLayout,
});

function DashboardLayout() {
  return (
    <AuthGuard>
      <SidebarWrapper>
        <Outlet />
        <CommandCenter />
        <HotkeyHelp />
        <GlobalModals />
      </SidebarWrapper>
    </AuthGuard>
  );
}
