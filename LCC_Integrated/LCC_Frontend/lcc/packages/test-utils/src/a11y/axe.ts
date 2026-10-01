/**
 * axe-core runner — runs axe against a rendered element or document.
 */

import axe, { type Result, type RunOptions } from 'axe-core';

export interface AxeRunOptions extends RunOptions {}

export async function runAxe(
  context: Element | Document = document,
  options: AxeRunOptions = {},
): Promise<Result[]> {
  const results = await axe.run(context, options);
  return results.violations;
}

export function expectNoViolations(violations: Result[]): void {
  if (violations.length > 0) {
    const messages = violations.map((v) => {
      const nodes = v.nodes.map((n) => n.target.join(' ')).join('\n  ');
      return `${v.id}: ${v.description}\n  ${nodes}\n  Help: ${v.helpUrl}`;
    });
    throw new Error(`Accessibility violations:\n${messages.join('\n\n')}`);
  }
}
