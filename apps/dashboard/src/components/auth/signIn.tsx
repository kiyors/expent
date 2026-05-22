"use client";

import { Button } from "@expent/ui/components/button";
import { toast } from "@expent/ui/components/goey-toaster";
import { InputGroup, InputGroupAddon, InputGroupInput } from "@expent/ui/components/input-group";
import { AtSignIcon, ChevronLeftIcon, KeyRoundIcon } from "lucide-react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useEffect, useState, useTransition } from "react";
import { AuthDivider } from "@/components/auth/auth-divider";
import { AuthShades } from "@/components/auth/auth-shades";
import { SocialLogins } from "@/components/auth/auth-social";
import { Logo } from "@/components/ui-elements/logo";
import { signIn, useSession } from "@/lib/auth-client";

export function SignIn() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const router = useRouter();
  const { data: session, isPending: isSessionPending } = useSession();
  const [_isTransitionPending, startTransition] = useTransition();

  useEffect(() => {
    if (!isSessionPending && session) {
      startTransition(() => {
        router.push("/");
      });
    }
  }, [session, isSessionPending, router]);

  const handleSignIn = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsLoading(true);
    const { data, error } = await signIn.email({
      email,
      password,
    });

    setIsLoading(false);
    if (error) {
      toast.error(error.message || "Failed to sign in");
    } else {
      window.location.href = "/";
    }
  };

  return (
    <div className="relative flex min-h-screen flex-col justify-center px-8">
      <AuthShades />
      <Button className="absolute top-7 left-5" variant="ghost" render={<Link href="/" />} nativeButton={false}>
        <ChevronLeftIcon data-icon="inline-start" />
        Home
      </Button>

      <div className="mx-auto gap-y-4 sm:w-sm">
        <Logo className="h-4.5 lg:hidden mx-auto" />
        <div className="flex flex-col gap-y-1 text-center">
          <h1 className="font-semibold text-2xl tracking-wide">Sign In or Join Now!</h1>
          <p className="text-base text-muted-foreground">login or create your expent account.</p>
        </div>

        <SocialLogins />

        <AuthDivider>OR</AuthDivider>

        <form className="gap-y-2 text-center" onSubmit={handleSignIn}>
          <p className="text-muted-foreground text-xs">Enter your credentials to sign in</p>
          <InputGroup>
            <InputGroupInput
              placeholder="your.email@example.com"
              type="email"
              required
              value={email}
              onChange={(e) => setEmail(e.target.value)}
            />
            <InputGroupAddon align="inline-start">
              <AtSignIcon />
            </InputGroupAddon>
          </InputGroup>

          <InputGroup>
            <InputGroupInput
              placeholder="Password"
              type="password"
              required
              value={password}
              onChange={(e) => setPassword(e.target.value)}
            />
            <InputGroupAddon align="inline-start">
              <KeyRoundIcon />
            </InputGroupAddon>
          </InputGroup>

          <Button className="w-full" type="submit" disabled={isLoading}>
            {isLoading ? "Signing in..." : "Continue With Email"}
          </Button>
        </form>

        <div className="flex flex-col gap-y-4 mt-8 text-center">
          <p className="text-muted-foreground text-sm">
            By clicking continue, you agree to our{" "}
            <a className="underline underline-offset-4 hover:text-primary" href="#">
              Terms of Service
            </a>{" "}
            and{" "}
            <a className="underline underline-offset-4 hover:text-primary" href="#">
              Privacy Policy
            </a>
            .
          </p>

          <p className="text-muted-foreground text-sm">
            New here?{" "}
            <Link className="font-semibold text-primary underline underline-offset-4" href="/sign-up">
              Create an account
            </Link>
          </p>
        </div>
      </div>
    </div>
  );
}
