/**
 * Toaster — ToastViewport + provider.
 */

import { ToastProvider, ToastViewport } from './toast';

export function Toaster(): React.ReactElement {
  return (
    <ToastProvider>
      <ToastViewport />
    </ToastProvider>
  );
}
