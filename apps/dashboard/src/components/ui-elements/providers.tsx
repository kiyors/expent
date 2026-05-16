"use client";

import { Toaster } from "@expent/ui/components/goey-toaster";
import { HotkeysProvider } from "@tanstack/react-hotkeys";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MotionConfig } from "motion/react";
import { useTheme } from "next-themes";
import { useState } from "react";
import { ThemeProvider } from "@/components/ui-elements/theme-provider";

function AppToaster() {
  const { resolvedTheme } = useTheme();
  return <Toaster theme={resolvedTheme === "dark" ? "dark" : "light"} position="bottom-right" closeButton />;
}

export function Providers({ children }: { children: React.ReactNode }) {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            staleTime: 1000 * 60 * 5, // 5 minutes
          },
        },
      }),
  );

  return (
    <QueryClientProvider client={queryClient}>
      <HotkeysProvider>
        <ThemeProvider attribute="class" defaultTheme="system" enableSystem disableTransitionOnChange>
          <MotionConfig reducedMotion="user">
            {children}
            <AppToaster />
          </MotionConfig>
        </ThemeProvider>
      </HotkeysProvider>
    </QueryClientProvider>
  );
}
