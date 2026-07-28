import { createServerFn } from "@tanstack/react-start";
import { getRequestHeaders } from "@tanstack/react-start/server";

import { authClient } from "@/lib/AuthClient";

export const getSession = createServerFn({ method: "GET" }).handler(async () => {
  const headers = getRequestHeaders();
  const { data: session } = await authClient.getSession({
    fetchOptions: {
      headers: {
        cookie: headers.get("cookie") || "",
      },
    },
  });

  return session;
});

export const ensureSession = createServerFn({ method: "GET" }).handler(async () => {
  const headers = getRequestHeaders();
  const { data: session } = await authClient.getSession({
    fetchOptions: {
      headers: {
        cookie: headers.get("cookie") || "",
      },
    },
  });

  if (!session) {
    throw new Error("Unauthorized");
  }

  return session;
});
