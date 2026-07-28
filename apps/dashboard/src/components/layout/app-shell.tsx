import { SidebarInset, SidebarProvider } from "@expent/ui/components/sidebar";

import { DashboardSkeleton } from "@/components/dashboard/dashboard-skeleton";
import { AppNavbar } from "@/components/layout/app-navbar";
import { AppSidebar } from "@/components/layout/app-sidebar";

export function AppShell() {
  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset>
        <AppNavbar />
        <div className="flex flex-1 flex-col gap-4 p-4 md:p-6">
          <DashboardSkeleton />
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}
