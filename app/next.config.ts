import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  /* config options here */
  reactCompiler: true,
  // Disable turbopack for now
  turbopack: {
    root: '/home/yash/zyncx/app',
  },
};

export default nextConfig;
