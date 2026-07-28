import { createFileRoute } from "@tanstack/react-router";

import { AuthPage } from "@/components/auth/AuthPage";
import { SignUp } from "@/components/auth/SignUp";

export const Route = createFileRoute("/sign-up")({
  component: () => (
    <AuthPage
      author="Sarah Jenkins"
      quote="Setting up my business profile was incredibly intuitive. The best onboarding experience I've had."
    >
      <SignUp />
    </AuthPage>
  ),
});
