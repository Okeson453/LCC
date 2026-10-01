# syntax=docker/dockerfile:1.7

# Multi-stage build for the Next.js web dashboard.

# ─── deps ────────────────────────────────────────────────────────────────
FROM node:20.12.2-alpine AS deps
WORKDIR /app

# pnpm
RUN corepack enable && corepack prepare pnpm@8.15.0 --activate

# Copy workspace manifests
COPY pnpm-workspace.yaml package.json ./
COPY tsconfig.base.json tsconfig.json ./
COPY packages/ packages/
COPY apps/web-dashboard/package.json ./apps/web-dashboard/

RUN if [ -f pnpm-lock.yaml ]; then pnpm install --frozen-lockfile; else pnpm install; fi

# ─── builder ─────────────────────────────────────────────────────────────
FROM node:20.12.2-alpine AS builder
WORKDIR /app

RUN corepack enable && corepack prepare pnpm@8.15.0 --activate

# Copy installed deps from previous stage
COPY --from=deps /app/node_modules ./node_modules
COPY --from=deps /app/apps/web-dashboard/node_modules ./apps/web-dashboard/node_modules

# Copy source
COPY . .

# Codegen (if schemas exist) — produces @lcc/api-types
RUN pnpm codegen || echo "codegen skipped"

ENV NEXT_TELEMETRY_DISABLED=1
ENV NODE_ENV=production

WORKDIR /app/apps/web-dashboard
RUN pnpm build

# ─── runner ──────────────────────────────────────────────────────────────
FROM node:20.12.2-alpine AS runner
WORKDIR /app

ENV NODE_ENV=production
ENV NEXT_TELEMETRY_DISABLED=1
ENV PORT=3000
ENV HOSTNAME=0.0.0.0

RUN addgroup --system --gid 1001 nodejs \
  && adduser --system --uid 1001 nextjs

COPY --from=builder /app/apps/web-dashboard/public ./apps/web-dashboard/public
COPY --from=builder --chown=nextjs:nodejs /app/apps/web-dashboard/.next/standalone ./
COPY --from=builder --chown=nextjs:nodejs /app/apps/web-dashboard/.next/static ./apps/web-dashboard/.next/static

USER nextjs

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:3000/api/health || exit 1

CMD ["node", "apps/web-dashboard/server.js"]
