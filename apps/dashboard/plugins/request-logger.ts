/* eslint-disable @typescript-eslint/no-explicit-any */
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

          const durationMs = performance.now() - start;
          const duration = durationMs.toFixed(2);
          const status = res.statusCode;

          // Determine request type
          const isApi = req.originalUrl?.startsWith("/api");
          const isAsset = req.originalUrl?.match(/\.(js|jsx|ts|tsx|css|svg|png|jpeg|jpg|woff|woff2|ico|json)(\?.*)?$/);

          // For a cleaner console, skip logging successful static assets
          if (isAsset && status < 400) {
            return;
          }

          // Type label and color
          let label = "\x1b[36m[router]\x1b[0m"; // Cyan
          if (isApi)
            label = "\x1b[35m[api]\x1b[0m"; // Magenta
          else if (isAsset) label = "\x1b[90m[asset]\x1b[0m"; // Gray

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

          // Colorize methods
          const methodColor =
            req.method === "GET"
              ? "\x1b[32m" // Green
              : req.method === "POST"
                ? "\x1b[33m" // Yellow
                : req.method === "PUT"
                  ? "\x1b[34m" // Blue
                  : req.method === "DELETE"
                    ? "\x1b[31m" // Red
                    : req.method === "PATCH"
                      ? "\x1b[35m" // Magenta
                      : "\x1b[36m"; // Cyan

          // Pad method to align URLs vertically (DELETE is 6 chars)
          const paddedMethod = (req.method || "").padEnd(6);

          // Dim the URL if it's just an asset, and always dim query strings
          const [path, query] = (req.originalUrl || "").split("?");
          let urlString = isAsset ? `\x1b[90m${path}\x1b[0m` : path;
          if (query) {
            urlString += `\x1b[90m?${query}\x1b[0m`;
          }

          // Highlight slow requests (yellow > 200ms, red > 500ms)
          const durationColor = durationMs > 500 ? "\x1b[31m" : durationMs > 200 ? "\x1b[33m" : "\x1b[90m";

          // Format payload size if available
          const contentLength = res.getHeader("content-length");
          let sizeString = "";
          if (contentLength) {
            const bytes = parseInt(String(contentLength), 10);
            if (!isNaN(bytes)) {
              const kb = (bytes / 1024).toFixed(1);
              sizeString = ` \x1b[90m${kb}kB\x1b[0m`;
            }
          }

          const time = new Date().toLocaleTimeString("en-US", {
            hour12: false,
            hour: "2-digit",
            minute: "2-digit",
            second: "2-digit",
          });

          console.log(
            `\x1b[90m${time}\x1b[0m ${label} ${statusColor}${status}\x1b[0m ${methodColor}${paddedMethod}\x1b[0m ${urlString} ${durationColor}${duration}ms\x1b[0m${sizeString}`,
          );
        });
        next();
      });
    },
  };
}
