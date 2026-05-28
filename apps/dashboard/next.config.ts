import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  /* config options here */
  output: "standalone",
  reactCompiler: true,
  outputFileTracingIncludes: {
    "/**": ["../../node_modules/mupdf/dist/*.wasm", "../../packages/wasm/pkg/*.wasm"],
  },
  experimental: {
    viewTransition: true,
  },
  turbopack: {
    resolveAlias: {
      "wasm_bg.wasm": "../../packages/wasm/pkg/wasm_bg.wasm",
      "mupdf-wasm.wasm": "../../node_modules/mupdf/dist/mupdf-wasm.wasm",
    },
  },
  webpack(config, { isServer: _isServer, webpack: _webpack }) {
    config.experiments = {
      ...config.experiments,
      asyncWebAssembly: true,
      layers: true,
    };

    return config;
  },
  async rewrites() {
    return [
      {
        source: "/api/:path*",
        destination: `${process.env.NEXT_PUBLIC_API_BASE_URL || "http://localhost:7878"}/api/:path*`,
      },
    ];
  },
};

export default nextConfig;
