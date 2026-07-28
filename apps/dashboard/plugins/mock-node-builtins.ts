import type { Plugin } from "vite";

export function mockNodeBuiltins(): Plugin {
  return {
    name: "mock-node-builtins",
    resolveId(id: string) {
      if (id === "node:fs" || id === "module" || id === "fs") {
        return id;
      }
    },
    load(id: string) {
      if (id === "node:fs" || id === "module" || id === "fs") {
        return "export default {};";
      }
    },
  };
}
