FROM node:20.11-bookworm-slim AS base
WORKDIR /app
ENV NODE_ENV=production \
    NEXT_TELEMETRY_DISABLED=1
RUN apt-get update && apt-get install -y --no-install-recommends dumb-init && rm -rf /var/lib/apt/lists/*

COPY . .
RUN corepack enable && if [ -f pnpm-lock.yaml ]; then pnpm install --frozen-lockfile; else pnpm install; fi
RUN pnpm -r --filter './packages/*' build && pnpm --filter @lcc/web-dashboard build

ENV PORT=3000
EXPOSE 3000
USER node
ENTRYPOINT ["dumb-init", "--"]
CMD ["pnpm", "--filter", "@lcc/web-dashboard", "start"]
