export const queryKeys = {
  contacts: {
    all: ["contacts"] as const,
    detail: (id: string) => ["contacts", id] as const,
    suggestions: ["contacts", "suggestions"] as const,
  },
  transactions: {
    all: ["transactions"] as const,
    page: (limit: number, offset: number) => ["transactions", "page", limit, offset] as const,
    summary: ["transactions", "summary"] as const,
  },
  wallets: { all: ["wallets"] as const },
  budgets: { all: ["budgets"] as const, health: ["budgets", "health"] as const },
  categories: { all: ["categories"] as const },
  p2p: {
    pending: ["p2p-pending"] as const,
    groups: ["groups"] as const,
    groupMembers: (id: string) => ["group-members", id] as const,
    ledgerTabs: ["ledger-tabs"] as const,
  },
  reconciliation: {
    all: ["reconciliation"] as const,
    matches: (rowId: string) => ["reconciliation", "matches", rowId] as const,
  },
  subscriptions: {
    all: ["subscriptions"] as const,
    detection: ["subscriptions", "detection"] as const,
  },
  demo: {
    status: ["demo", "status"] as const,
  },
} as const;
