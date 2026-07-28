import { createFileRoute } from "@tanstack/react-router";
import { SignIn } from "@/components/auth/signIn";

export const Route = createFileRoute("/sign-in")({
  component: SignIn,
  validateSearch: (search: Record<string, unknown>): { redirect?: string } => {
    return {
      redirect: search.redirect as string | undefined,
    };
  },
});
