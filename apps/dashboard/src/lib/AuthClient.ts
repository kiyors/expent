import { passkeyClient } from "@better-auth/passkey/client";
import { usernameClient } from "better-auth/client/plugins";
import { createAuthClient } from "better-auth/react";

/**
 * Client-side authentication client.
 * Pointing to the Rust server's /api/auth endpoints.
 */
export const authClient = createAuthClient({
  // Use absolute URL to the Rust backend (7878) for both client and server.
  // On server, use 127.0.0.1 instead of localhost to prevent Node 18+ IPv6 fetch failures.
  baseURL:
    import.meta.env.VITE_API_BASE_URL ||
    (typeof window !== "undefined" ? "http://localhost:7878" : "http://127.0.0.1:7878"),
  plugins: [passkeyClient(), usernameClient()],
});

export const { signIn, signUp, useSession, signOut } = authClient;
