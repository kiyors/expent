import { passkeyClient } from "@better-auth/passkey/client";
import { usernameClient } from "better-auth/client/plugins";
import { createAuthClient } from "better-auth/react";

/**
 * Client-side authentication client.
 * Pointing to the Rust server's /api/auth endpoints.
 */
export const authClient = createAuthClient({
  baseURL: `${process.env.NEXT_PUBLIC_API_BASE_URL || "http://localhost:7878"}/api/auth`,
  plugins: [passkeyClient(), usernameClient()],
});

export const { signIn, signUp, useSession, signOut } = authClient;
