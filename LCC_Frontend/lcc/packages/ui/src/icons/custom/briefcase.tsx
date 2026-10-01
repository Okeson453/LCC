import * as React from 'react';

export function BriefcaseIcon({
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
      <path d="M20 6h-4V4a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v2H4a2 2 0 0 0-2 2v11a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2M10 4h4v2h-4V4m10 15H4V8h16v11Z" />
    </svg>
  );
}
