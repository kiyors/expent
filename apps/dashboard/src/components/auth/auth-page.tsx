"use client";

import type React from "react";
import { FloatingPaths } from "@/components/ui-elements/floating-paths";
import { Logo } from "@/components/ui-elements/logo";

interface AuthPageProps {
  children: React.ReactNode;
  quote?: string;
  author?: string;
}

export function AuthPage({
  children,
  quote = "This Platform has helped me to save time and serve my clients faster than ever before.",
  author = "Ali Hassan",
}: AuthPageProps) {
  return (
    <main className="relative md:h-screen md:overflow-hidden lg:grid lg:grid-cols-2">
      {/* Branding and Quote Side - Always on left */}
      <div className="relative hidden h-full flex-col border-r bg-secondary p-10 lg:flex dark:bg-secondary/20">
        <div className="absolute inset-0 bg-linear-to-b from-transparent via-transparent to-background" />
        <Logo className="mr-auto h-4.5" />

        <div className="z-10 mt-auto">
          <blockquote className="gap-y-2">
            <p className="text-xl">&ldquo;{quote}&rdquo;</p>
            <footer className="font-mono font-semibold text-sm">~ {author}</footer>
          </blockquote>
        </div>
        <div className="absolute inset-0">
          <FloatingPaths position={1} />
          <FloatingPaths position={-1} />
        </div>
      </div>
      <div className="h-full">{children}</div>
    </main>
  );
}
