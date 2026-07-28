import { SidebarClient } from "@/components/layout/sidebar-client";
import { useEffect, useState } from "react";

export function SidebarWrapper({ children }: { children: React.ReactNode }) {
  const [defaultOpen, setDefaultOpen] = useState(true);

  useEffect(() => {
    const isClient = typeof window !== "undefined";
    if (isClient) {
      const match = document.cookie.match(new RegExp("(^| )sidebar_state=([^;]+)"));
      if (match) {
        setDefaultOpen(match[2] !== "false");
      }
    }
  }, []);

  return <SidebarClient defaultOpen={defaultOpen}>{children}</SidebarClient>;
}
