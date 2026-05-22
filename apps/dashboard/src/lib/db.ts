import type { Budget, Category, Contact, PaginatedTransactions, Transaction, Wallet } from "@expent/types";
import { BTreeIndex, createCollection, localStorageCollectionOptions } from "@tanstack/db";
import { api } from "./api-client";

// In @tanstack/db v0.6.5, we export an object with collections.
// We use localStorageCollectionOptions to handle persistence and cross-tab sync.

const walletOptions = localStorageCollectionOptions({
  storageKey: "expent_wallets",
  getKey: (wallet: Wallet) => wallet.id,
});

const transactionsOptions = localStorageCollectionOptions({
  storageKey: "expent_transactions",
  getKey: (txn: Transaction) => txn.id,
  defaultIndexType: BTreeIndex,
});

const budgetOptions = localStorageCollectionOptions({
  storageKey: "expent_budgets",
  getKey: (budget: Budget) => budget.id,
});

const categoryOptions = localStorageCollectionOptions({
  storageKey: "expent_categories",
  getKey: (cat: Category) => cat.id,
});

const contactOptions = localStorageCollectionOptions({
  storageKey: "expent_contacts",
  getKey: (contact: Contact) => contact.id,
});

export const db = {
  wallets: createCollection({
    ...walletOptions,
    sync: {
      sync: (params) => {
        walletOptions.sync.sync(params);
        api
          .get<Wallet[]>("/api/wallets")
          .then((wallets) => {
            params.begin();
            for (const wallet of wallets) {
              params.write({ type: "insert", value: wallet });
            }
            params.commit();
          })
          .catch((error) => console.error("Failed to sync wallets:", error));
      },
    },
  }),
  transactions: createCollection({
    ...transactionsOptions,
    sync: {
      sync: (params) => {
        transactionsOptions.sync.sync(params);
        api
          .get<PaginatedTransactions>("/api/transactions?limit=100")
          .then((res) => {
            params.begin();
            for (const txn of res.items) {
              params.write({ type: "insert", value: txn });
            }
            params.commit();
          })
          .catch((error) => console.error("Failed to sync transactions:", error));
      },
    },
  }),
  budgets: createCollection({
    ...budgetOptions,
    sync: {
      sync: (params) => {
        budgetOptions.sync.sync(params);
        api
          .get<Budget[]>("/api/budgets")
          .then((budgets) => {
            params.begin();
            for (const budget of budgets) {
              params.write({ type: "insert", value: budget });
            }
            params.commit();
          })
          .catch((error) => console.error("Failed to sync budgets:", error));
      },
    },
  }),
  categories: createCollection({
    ...categoryOptions,
    sync: {
      sync: (params) => {
        categoryOptions.sync.sync(params);
        api
          .get<Category[]>("/api/categories")
          .then((categories) => {
            params.begin();
            for (const cat of categories) {
              params.write({ type: "insert", value: cat });
            }
            params.commit();
          })
          .catch((error) => console.error("Failed to sync categories:", error));
      },
    },
  }),
  contacts: createCollection({
    ...contactOptions,
    sync: {
      sync: (params) => {
        contactOptions.sync.sync(params);
        api
          .get<Contact[]>("/api/contacts")
          .then((contacts) => {
            params.begin();
            for (const contact of contacts) {
              params.write({ type: "insert", value: contact });
            }
            params.commit();
          })
          .catch((error) => console.error("Failed to sync contacts:", error));
      },
    },
  }),
};

// Add explicit index for performance
db.transactions.createIndex((row) => row.date);
