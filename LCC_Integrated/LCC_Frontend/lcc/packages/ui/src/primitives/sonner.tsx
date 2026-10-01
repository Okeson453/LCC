/**
 * Sonner-style toaster — alternative to Radix toast.
 */

import { Toaster as SonnerToaster } from 'sonner';

export interface ToasterProps {
  theme?: 'light' | 'dark' | 'system';
  position?: 'top-left' | 'top-center' | 'top-right' | 'bottom-left' | 'bottom-center' | 'bottom-right';
  richColors?: boolean;
  closeButton?: boolean;
  expand?: boolean;
}

export function Toaster(props: ToasterProps): React.ReactElement {
  return (
    <SonnerToaster
      theme={props.theme ?? 'dark'}
      position={props.position ?? 'bottom-right'}
      richColors={props.richColors ?? true}
      closeButton={props.closeButton ?? true}
      expand={props.expand ?? false}
    />
  );
}
