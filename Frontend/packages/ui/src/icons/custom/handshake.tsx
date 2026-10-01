import * as React from 'react';

export function HandshakeIcon({
  className,
  size = 24,
  ...props
}: React.SVGProps<SVGSVGElement> & { size?: number }): React.ReactElement {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="currentColor"
      className={className}
      aria-hidden="true"
      {...props}
    >
      <path d="M21.71 8.29l-4-4a1 1 0 0 0-1.42 0l-4 4-1 1-3 3-1 1-3 3a1 1 0 0 0 0 1.42l4 4a1 1 0 0 0 1.42 0l3-3 1-1 3-3 1-1 4-4a1 1 0 0 0 0-1.42M14.5 14.5L11 18l-3-3 3.5-3.5 3 3m5-5l-3 3-3-3 3-3 3 3Z" />
    </svg>
  );
}
