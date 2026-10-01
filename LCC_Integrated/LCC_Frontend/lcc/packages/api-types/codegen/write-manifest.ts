/**
 * Codegen: write-manifest.ts
 *
 * Emits `packages/api-types/manifest.json` describing when the generated
 * files were last regenerated and the source files they were generated from.
 * CI checks this manifest to ensure generated files are in sync.
 */

import { readFileSync, writeFileSync, statSync } from 'node:fs';
import { join } from 'node:path';

const ROOT = join(import.meta.dirname, '../../..');
const SOURCES = [
  join(ROOT, 'schemas/openapi/api-gateway.yaml'),
  join(ROOT, 'schemas/events/member.created.schema.json'),
  join(ROOT, 'schemas/events/content.item.approved.schema.json'),
  join(ROOT, 'schemas/events/compliance.restriction_detected.schema.json'),
  join(ROOT, 'proto/gen/typescript'),
];

const MANIFEST_PATH = join(ROOT, 'packages/api-types/manifest.json');

function main() {
  const sources: Array<{ path: string; modified_at: string; sha256?: string }> = [];
  for (const p of SOURCES) {
    try {
      const stat = statSync(p);
      sources.push({
        path: p.replace(ROOT, ''),
        modified_at: stat.mtime.toISOString(),
      });
    } catch {
      sources.push({ path: p.replace(ROOT, ''), modified_at: 'missing' });
    }
  }

  const manifest = {
    generated_at: new Date().toISOString(),
    sources,
    codegen_version: '1.0.0',
  };

  writeFileSync(MANIFEST_PATH, JSON.stringify(manifest, null, 2) + '\n', 'utf-8');
  console.log(`[write-manifest] wrote ${MANIFEST_PATH}`);
}

main();
