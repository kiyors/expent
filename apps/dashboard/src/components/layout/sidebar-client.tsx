import { SidebarInset, SidebarProvider } from "@expent/ui/components/sidebar";

import { AppNavbar } from "@/components/layout/app-navbar";
import { AppSidebar } from "@/components/layout/app-sidebar";

export function SidebarClient({ defaultOpen, children }: { defaultOpen: boolean; children: React.ReactNode }) {
  return (
    <SidebarProvider defaultOpen={defaultOpen}>
      <AppSidebar />
      <SidebarInset>
        <AppNavbar />
        <div className="flex flex-1 flex-col">{children}</div>
      </SidebarInset>
    </SidebarProvider>
  );
}
