import type { Plugin } from "vite";

export function requestLogger(): Plugin {
  return {
    name: "vite-plugin-request-logging",
    configureServer(server: any) {
      server.middlewares.use((req: any, res: any, next: any) => {
        const start = performance.now();
        res.on("finish", () => {
          // Ignore polling or HMR requests to keep it clean
          if (
            req.originalUrl?.includes("?import") ||
            req.originalUrl?.includes("vite_ping") ||
            req.originalUrl?.includes("@fs") ||
            req.originalUrl?.includes("/@vite/client") ||
            req.originalUrl?.includes("/@react-refresh")
          ) {
            return;
          }

          const duration = (performance.now() - start).toFixed(2);
          const status = res.statusCode;

          // Colorize status codes for better readability
          const statusColor =
            status >= 500
              ? "\x1b[31m" // Red
              : status >= 400
                ? "\x1b[33m" // Yellow
                : status >= 300
                  ? "\x1b[36m" // Cyan
                  : status >= 200
                    ? "\x1b[32m" // Green
                    : "\x1b[0m"; // Reset

          console.log(
            `\x1b[90m[Vite]\x1b[0m ${req.method} ${statusColor}${status}\x1b[0m ${req.originalUrl} \x1b[90m- ${duration}ms\x1b[0m`,
          );
        });
        next();
      });
    },
  };
}
