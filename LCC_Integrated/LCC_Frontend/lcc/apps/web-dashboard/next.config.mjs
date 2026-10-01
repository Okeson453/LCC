/**
 * Next.js configuration. Plain object export — Next 14 does not provide
 * `defineConfig`; that was an early scaffolding artifact fixed per audit
 * finding C-01.
 *
 * @type {import('next').NextConfig}
 */
const nextConfig = {
  reactStrictMode: true,
  poweredByHeader: false,
  compress: true,
  productionBrowserSourceMaps: false,

  // Performance budgets — enforced via bundle-size-check.sh + Lighthouse CI.
  experimental: {
    optimizePackageImports: ['lucide-react', '@lcc/ui'],
  },

  // Security: CSP uses nonces in scripts (non-essential inline is forbidden
  // in prod; 'unsafe-inline' is only retained for dev hot-reload via the
  // dev-only fallback in the runbook at docs/frontend/runbooks/csp-rollback.md).
  async headers() {
    const isProd = process.env.NODE_ENV === 'production';
    const scriptSrc = isProd
      ? "'self' 'nonce-{NONCE}' https://*.lcc.okeson.example"
      : "'self' 'unsafe-inline' 'unsafe-eval' https://*.lcc.okeson.example";

    return [
      {
        source: '/:path*',
        headers: [
          { key: 'X-Frame-Options', value: 'DENY' },
          { key: 'X-Content-Type-Options', value: 'nosniff' },
          { key: 'Referrer-Policy', value: 'strict-origin-when-cross-origin' },
          { key: 'Permissions-Policy', value: 'camera=(), microphone=(), geolocation=()' },
          {
            key: 'Strict-Transport-Security',
            value: 'max-age=63072000; includeSubDomains; preload',
          },
          {
            key: 'Content-Security-Policy',
            value: [
              "default-src 'self'",
              `script-src ${scriptSrc}`,
              "style-src 'self' 'unsafe-inline'",
              "img-src 'self' data: blob: https://media.licdn.com https://*.lcc.okeson.example",
              "font-src 'self' data:",
              "connect-src 'self' https://api.lcc.okeson.example wss://api.lcc.okeson.example",
              "frame-ancestors 'none'",
              "base-uri 'self'",
              "form-action 'self'",
            ].join('; '),
          },
        ],
      },
    ];
  },

  // Rewrites: only auth routes need rewrites; the rest goes through the k8s
  // ingress / Edge proxy, which attaches trace_id and Idempotency-Key.
  async rewrites() {
    const base = process.env.NEXT_PUBLIC_API_BASE ?? 'http://localhost:8080';
    return [
      { source: '/api/v1/auth/:path*', destination: `${base}/api/v1/auth/:path*` },
      { source: '/api/v1/extensions/exchange', destination: `${base}/api/v1/extensions/exchange` },
    ];
  },

  images: {
    remotePatterns: [
      { protocol: 'https', hostname: 'media.licdn.com', pathname: '/**' },
      { protocol: 'https', hostname: '*.licdn.com', pathname: '/**' },
      { protocol: 'https', hostname: '*.lcc.okeson.example' },
    ],
    formats: ['image/avif', 'image/webp'],
    deviceSizes: [640, 750, 828, 1080, 1200, 1920],
    imageSizes: [16, 32, 48, 64, 96, 128, 256, 384],
    minimumCacheTTL: 60 * 60 * 24 * 7, // 7 days
  },

  output: 'standalone',
};

export default nextConfig;
