/**
 * PDF generation — client-side preview for proposals and applications.
 *
 * Production-grade: uses `print` stylesheet for now (browser-driven PDF).
 * Future enhancement: integrate with `react-pdf` or `@react-pdf/renderer`.
 */

export function printAsPdf(elementId: string, filename = 'document.pdf'): void {
  if (typeof window === 'undefined') return;
  const originalTitle = document.title;
  document.title = filename;
  const content = document.getElementById(elementId);
  if (!content) return;
  content.classList.add('print-mode');
  window.print();
  content.classList.remove('print-mode');
  document.title = originalTitle;
}
