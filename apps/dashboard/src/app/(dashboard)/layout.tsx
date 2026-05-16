import { AuthGuard } from "@/components/auth/auth-guard";
import { CommandCenter } from "@/components/layout/command-center";
import { GlobalModals } from "@/components/layout/global-modals";
import { HotkeyHelp } from "@/components/layout/hotkey-help";
import { SidebarWrapper } from "@/components/layout/sidebar-wrapper";

export default function DashboardLayout({ children }: { children: React.ReactNode }) {
  return (
    <AuthGuard>
      <SidebarWrapper>
        {children}
        <CommandCenter />
        <HotkeyHelp />
        <GlobalModals />
      </SidebarWrapper>
    </AuthGuard>
  );
}
