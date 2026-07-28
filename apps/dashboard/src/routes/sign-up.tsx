import { createFileRoute } from "@tanstack/react-router";
import { SignUp } from "@/components/auth/signUp";

export const Route = createFileRoute("/sign-up")({
  component: SignUp,
});
