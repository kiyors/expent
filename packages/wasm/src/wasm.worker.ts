import init, { detect_subscription_patterns, generate_dashboard_summary } from "../pkg/wasm";

self.onmessage = async (e: MessageEvent) => {
  const { type, payload } = e.data;

  // Initialize WASM in the worker
  await init();

  let result: unknown;
  switch (type) {
    case "DETECT_SUBSCRIPTIONS":
      result = detect_subscription_patterns(payload.transactions);
      break;
    case "GENERATE_SUMMARY":
      result = generate_dashboard_summary(payload.transactions, payload.wallets, payload.categories);
      break;
    default:
      console.error("Unknown task type:", type);
      return;
  }

  postMessage({ type, result });
};
