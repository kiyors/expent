import { authClient } from "./auth-client";

// No need for NEXT_PUBLIC_API_BASE_URL for client requests if using rewrites
// For server-side requests (if any), it still needs an absolute URL.
const API_URL = typeof window === "undefined" ? process.env.NEXT_PUBLIC_API_BASE_URL || "http://localhost:7878" : "";

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const session = await authClient.getSession();
  const token = session?.data?.session.token;

  const headers = new Headers(options.headers);
  if (token) {
    headers.set("Authorization", `Bearer ${token}`);
  }
  if (options.body && !(options.body instanceof FormData)) {
    headers.set("Content-Type", "application/json");
  }

  // Ensure path starts with /
  const normalizedPath = path.startsWith("/") ? path : `/${path}`;

  const res = await fetch(`${API_URL}${normalizedPath}`, {
    ...options,
    headers,
  });

  if (!res.ok) {
    const error = await res.json().catch(() => ({ message: "Unknown error" }));
    throw new Error(error.message || res.statusText);
  }

  // Handle 204 No Content
  if (res.status === 204) {
    return {} as T;
  }

  return res.json();
}

export const api = {
  get: <T>(path: string, options?: RequestInit) => request<T>(path, { ...options, method: "GET" }),
  post: <T, B = any>(path: string, body?: B, options?: RequestInit) =>
    request<T>(path, { ...options, method: "POST", body: body ? JSON.stringify(body) : undefined }),
  put: <T, B = any>(path: string, body?: B, options?: RequestInit) =>
    request<T>(path, { ...options, method: "PUT", body: body ? JSON.stringify(body) : undefined }),
  patch: <T, B = any>(path: string, body?: B, options?: RequestInit) =>
    request<T>(path, { ...options, method: "PATCH", body: body ? JSON.stringify(body) : undefined }),
  delete: <T>(path: string, options?: RequestInit) => request<T>(path, { ...options, method: "DELETE" }),
};
