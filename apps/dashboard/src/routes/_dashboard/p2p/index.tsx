import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute("/_dashboard/p2p/")({
  beforeLoad: () => {
    throw redirect({
      to: "/p2p/pending",
    });
  },
});
