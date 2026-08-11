import type { NextConfig } from "next";
import path from "path";

const nextConfig: NextConfig = {
  turbopack: {
    // Pin the Turbopack workspace root to this app. Without it, Next.js infers
    // the root from the surrounding monorepo (the Cargo workspace at the repo
    // root) and fails to resolve the Next.js package during dev recompiles.
    root: path.resolve(__dirname),
  },
};

export default nextConfig;
