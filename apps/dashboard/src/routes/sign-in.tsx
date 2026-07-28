import { createFileRoute } from "@tanstack/react-router";
import { SignIn } from "@/components/auth/signIn";
import { AuthPage } from "@/components/auth/auth-page";

export const Route = createFileRoute("/sign-in")({
  component: () => (
    <AuthPage
      author="Ali Hassan"
      quote="This Platform has helped me to save time and serve my clients faster than ever before."
    >
      <SignIn />
    </AuthPage>
  ),
  validateSearch: (search: Record<string, unknown>): { redirect?: string } => {
    return {
      redirect: search.redirect as string | undefined,
    };
  },
});
