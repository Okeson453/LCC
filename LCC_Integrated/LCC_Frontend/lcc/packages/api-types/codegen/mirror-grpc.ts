/**
 * Codegen: mirror-grpc.ts
 *
 * Mirrors the TypeScript gRPC stubs from `proto/gen/typescript/` into
 * `packages/api-types/src/generated/grpc/`. The proto codegen step produces
 * the source files; this script maintains a stable frontend import path.
 */

import { readdirSync, readFileSync, writeFileSync, mkdirSync, existsSync } from 'node:fs';
import { join, dirname } from 'node:path';

const ROOT = join(import.meta.dirname, '../../..');
const SOURCE = join(ROOT, 'proto/gen/typescript');
const TARGET = join(ROOT, 'packages/api-types/src/generated/grpc');

function copyDir(src: string, dst: string) {
  if (!existsSync(dst)) mkdirSync(dst, { recursive: true });
  const entries = readdirSync(src, { withFileTypes: true });
  for (const entry of entries) {
    const srcPath = join(src, entry.name);
    const dstPath = join(dst, entry.name);
    if (entry.isDirectory()) {
      copyDir(srcPath, dstPath);
    } else if (entry.isFile() && entry.name.endsWith('.ts')) {
      const content = readFileSync(srcPath, 'utf-8');
      writeFileSync(dstPath, content, 'utf-8');
      console.log(`[mirror-grpc] ${srcPath} → ${dstPath}`);
    }
  }
}

if (!existsSync(SOURCE)) {
  console.warn(`[mirror-grpc] source not found: ${SOURCE} — skipping`);
  process.exit(0);
}

copyDir(SOURCE, TARGET);
console.log('[mirror-grpc] done');
