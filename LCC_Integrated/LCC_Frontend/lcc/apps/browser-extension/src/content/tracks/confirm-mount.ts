/**
 * Track B · ConfirmPrompt mount.
 */
import { createRoot, type Root } from 'react-dom/client';
import { ConfirmPrompt } from '../components/ConfirmPrompt';

const MOUNT_ATTR = 'data-lcc-confirm';

const openStates = new WeakMap<HTMLElement, { open: boolean; setOpen: (b: boolean) => void }>();

export function mountConfirmPrompt(target: HTMLElement, message: string): { resolve: (ok: boolean) => void } {
  if (target.hasAttribute(MOUNT_ATTR)) {
    return { resolve: () => undefined };
  }
  target.setAttribute(MOUNT_ATTR, '1');
  const host = document.createElement('div');
  target.appendChild(host);
  const root: Root = createRoot(host);

  return new Promise<boolean>((res) => {
    const render = (open: boolean): void => root.render(
      <ConfirmPrompt
        open={open}
        message={message}
        onConfirm={() => { render(false); root.unmount(); host.remove(); target.removeAttribute(MOUNT_ATTR); res(true); }}
        onCancel={() => { render(false); root.unmount(); host.remove(); target.removeAttribute(MOUNT_ATTR); res(false); }}
      />,
    );
    render(true);
    openStates.set(target, { open: true, setOpen: render });
  }) as unknown as { resolve: (ok: boolean) => void };
}
