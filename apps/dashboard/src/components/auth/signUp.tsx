"use client";

import { Button } from "@expent/ui/components/button";
import { toast } from "@expent/ui/components/goey-toaster";
import { InputGroup, InputGroupAddon, InputGroupInput } from "@expent/ui/components/input-group";
import { AtSignIcon, ChevronLeftIcon, KeyRoundIcon, UserIcon } from "lucide-react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useEffect, useState, useTransition } from "react";
import { AuthDivider } from "@/components/auth/auth-divider";
import { AuthShades } from "@/components/auth/auth-shades";
import { SocialLogins } from "@/components/auth/auth-social";
import { Logo } from "@/components/ui-elements/logo";
import { signUp, useSession } from "@/lib/auth-client";

export function SignUp() {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [name, setName] = useState("");
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

  const handleSignUp = async (e: React.FormEvent) => {
    e.preventDefault();
    if (password !== confirmPassword) {
      toast.error("Passwords do not match");
      return;
    }

    setIsLoading(true);
    const { data, error } = await signUp.email({
      email,
      password,
      name,
    });

    setIsLoading(false);
    if (error) {
      toast.error(error.message || "Failed to sign up");
    } else {
      window.location.href = "/";
    }
  };

  return (
    <div className="relative flex min-h-screen flex-col justify-center px-8">
      <AuthShades variant="flipped" />
      <Button className="absolute top-7 left-5" variant="ghost" render={<Link href="/" />} nativeButton={false}>
        <ChevronLeftIcon data-icon="inline-start" />
        Home
      </Button>

      <div className="mx-auto gap-y-4 sm:w-sm">
        <Logo className="h-4.5 lg:hidden mx-auto" />
        <div className="flex flex-col gap-y-1 text-center">
          <h1 className="font-semibold text-2xl tracking-wide">Create your account</h1>
          <p className="text-sm text-muted-foreground">Enter your details below to create your account</p>
        </div>

        <form className="gap-y-2" onSubmit={handleSignUp}>
          <InputGroup>
            <InputGroupInput
              placeholder="Name"
              type="text"
              required
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
            <InputGroupAddon align="inline-start">
              <UserIcon />
            </InputGroupAddon>
          </InputGroup>

          <InputGroup>
            <InputGroupInput
              placeholder="m@example.com"
              type="email"
              required
              value={email}
              onChange={(e) => setEmail(e.target.value)}
            />
            <InputGroupAddon align="inline-start">
              <AtSignIcon />
            </InputGroupAddon>
          </InputGroup>

          <div className="grid grid-cols-2 gap-2">
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
            <InputGroup>
              <InputGroupInput
                placeholder="Confirm Password"
                type="password"
                required
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
              />
              <InputGroupAddon align="inline-start">
                <KeyRoundIcon />
              </InputGroupAddon>
            </InputGroup>
          </div>

          <Button className="w-full" type="submit" disabled={isLoading}>
            {isLoading ? "Creating account..." : "Create account"}
          </Button>
        </form>

        <AuthDivider>OR CONTINUE WITH</AuthDivider>

        <SocialLogins />

        <div className="flex flex-col gap-y-4 mt-8 text-center">
          <p className="text-muted-foreground text-sm">
            By signing up, you agree to our{" "}
            <a className="underline underline-offset-4 hover:text-primary" href="#">
              Terms
            </a>{" "}
            and{" "}
            <a className="underline underline-offset-4 hover:text-primary" href="#">
              Privacy
            </a>
            .
          </p>

          <p className="text-muted-foreground text-sm">
            Already have an account?{" "}
            <Link className="font-semibold text-primary underline underline-offset-4" href="/sign-in">
              Sign in
            </Link>
          </p>
        </div>
      </div>
    </div>
  );
}
