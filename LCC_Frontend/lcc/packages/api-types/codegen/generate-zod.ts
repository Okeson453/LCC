/**
 * Codegen: generate-zod.ts
 *
 * Generates Zod schemas from OpenAPI schemas annotated with `x-zod` extensions.
 * If a schema does not have `x-zod`, falls back to a passthrough.
 *
 * In production, prefer keeping the hand-written `runtime/zod-schemas.ts` —
 * this script is a convenience for adding new schemas incrementally.
 */

import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { parse as parseYaml } from 'yaml';

const ROOT = join(import.meta.dirname, '../../..');
const OPENAPI_PATH = join(ROOT, 'schemas/openapi/api-gateway.yaml');
const OUTPUT_PATH = join(ROOT, 'packages/api-types/src/runtime/zod-schemas.ts');

function yamlTypeToZod(type: string, format?: string): string {
  if (type === 'string') {
    if (format === 'uuid') return 'z.string().uuid()';
    if (format === 'date-time') return 'z.string().datetime({ offset: true })';
    if (format === 'email') return 'z.string().email()';
    if (format === 'uri') return 'z.string().url()';
    return 'z.string()';
  }
  if (type === 'integer' || type === 'number') {
    return 'z.number()';
  }
  if (type === 'boolean') return 'z.boolean()';
  return 'z.unknown()';
}

function main() {
  const openapiText = readFileSync(OPENAPI_PATH, 'utf-8');
  const spec = parseYaml(openapiText) as {
    components?: {
      schemas?: Record<string, {
        type?: string;
        format?: string;
        enum?: string[];
        properties?: Record<string, unknown>;
        required?: string[];
        items?: { type?: string; format?: string; $ref?: string };
      }>;
    };
  };
  const schemas = spec.components?.schemas ?? {};

  let out = `// @generated - regenerate via pnpm codegen
import { z } from 'zod';

const uuid = z.string().uuid();
const dateTime = z.string().datetime({ offset: true });

`;

  for (const [name, schema] of Object.entries(schemas)) {
    const lines: string[] = [];
    const fields: string[] = [];
    for (const [fieldName, fieldSchemaRaw] of Object.entries(schema.properties ?? {})) {
      const fieldSchema = fieldSchemaRaw as { type?: string; format?: string; enum?: string[]; $ref?: string };
      const optional = !(schema.required ?? []).includes(fieldName);
      let zodType: string;
      if (fieldSchema.$ref) {
        const refName = fieldSchema.$ref.split('/').pop() ?? '';
        zodType = `${refName}Schema`;
      } else if (fieldSchema.enum) {
        zodType = `z.enum([${fieldSchema.enum.map((v) => JSON.stringify(v)).join(', ')}])`;
      } else {
        zodType = yamlTypeToZod(fieldSchema.type ?? 'string', fieldSchema.format);
      }
      fields.push(`  ${fieldName}: ${zodType}${optional ? '.optional()' : ''}`);
    }
    lines.push(`export const ${name}Schema = z.object({`);
    lines.push(...fields);
    lines.push('});');
    lines.push('');
    out += lines.join('\n') + '\n\n';
  }

  writeFileSync(OUTPUT_PATH, out, 'utf-8');
  console.log(`[generate-zod] wrote ${OUTPUT_PATH}`);
}

main();
