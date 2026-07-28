import { passkeyClient } from "@better-auth/passkey/client";
import { usernameClient } from "better-auth/client/plugins";
import { createAuthClient } from "better-auth/react";

/**
 * Client-side authentication client.
 * Pointing to the Rust server's /api/auth endpoints.
 */
export const authClient = createAuthClient({
  baseURL: import.meta.env.VITE_API_BASE_URL || "http://localhost:3000",
  plugins: [passkeyClient(), usernameClient()],
});

export const { signIn, signUp, useSession, signOut } = authClient;
