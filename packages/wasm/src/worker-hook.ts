import { useCallback, useEffect, useRef } from "react";

export function useWasmWorker() {
  const workerRef = useRef<Worker | null>(null);

  useEffect(() => {
    // We use a worker relative to this file's location.
    // Modern bundlers (Vite/Webpack 5/Turbopack) will bundle this correctly.
    workerRef.current = new Worker(new URL("./wasm.worker.ts", import.meta.url));
    return () => {
      workerRef.current?.terminate();
    };
  }, []);

  const runTask = useCallback(<T>(type: string, payload: unknown): Promise<T> => {
    return new Promise((resolve, reject) => {
      if (!workerRef.current) {
        reject(new Error("Worker not initialized"));
        return;
      }

      const handler = (e: MessageEvent) => {
        if (e.data.type === type) {
          workerRef.current?.removeEventListener("message", handler);
          resolve(e.data.result);
        }
      };

      workerRef.current.addEventListener("message", handler);
      workerRef.current.postMessage({ type, payload });
    });
  }, []);

  return { runTask };
}
