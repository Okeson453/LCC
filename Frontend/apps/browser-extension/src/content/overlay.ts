/**
 * Read-only overlay: surfaces next-best-action without performing any
 * side-effect action on the LinkedIn DOM. Tier 1 read actions only.
 */
export function mountReadOverlay(root: HTMLElement): void {
  const host = document.createElement('div');
  host.className = 'lcc-ext-read-overlay';
  host.setAttribute('role', 'complementary');
  host.setAttribute('aria-label', 'LinkedIn Manager suggestion');
  host.innerHTML = `
    <button type="button" class="lcc-ext-read-button"
      style="background:#0a66c2;color:#fff;border-radius:8px;padding:8px 12px;border:0;font-size:12px;cursor:pointer;box-shadow:0 2px 6px rgba(0,0,0,0.15);">
      View suggestion
    </button>
  `;
  root.appendChild(host);
  host.querySelector('button')?.addEventListener('click', () => {
    void chrome.runtime.sendMessage({ type: 'lcc.openSidePanel' });
  });
}
