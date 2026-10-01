'use client';

import * as React from 'react';
import { cn } from '../utils';

export interface VirtualListProps<T> {
  items: T[];
  rowHeight: number;
  height: number;
  renderRow: (item: T, index: number) => React.ReactNode;
  className?: string;
  ariaLabel?: string;
}

/**
 * VirtualList — keeps DOM nodes bounded for large lists (P-07).
 * For lists beyond ~100 items. Uses overscan=4.
 */
export function VirtualList<T>({ items, rowHeight, height, renderRow, className, ariaLabel }: VirtualListProps<T>): React.ReactElement {
  const [scrollTop, setScrollTop] = React.useState(0);
  const totalHeight = items.length * rowHeight;
  const startIndex = Math.max(0, Math.floor(scrollTop / rowHeight) - 4);
  const endIndex = Math.min(items.length, Math.ceil((scrollTop + height) / rowHeight) + 4);

  const onScroll = (e: React.UIEvent<HTMLDivElement>): void => {
    setScrollTop(e.currentTarget.scrollTop);
  };

  return (
    <div
      role="list"
      aria-label={ariaLabel ?? 'list'}
      onScroll={onScroll}
      className={cn('overflow-auto', className)}
      style={{ height }}
    >
      <div style={{ height: totalHeight, position: 'relative' }}>
        {items.slice(startIndex, endIndex).map((item, i) => {
          const index = startIndex + i;
          return (
            <div
              key={index}
              role="listitem"
              style={{ position: 'absolute', top: index * rowHeight, height: rowHeight, left: 0, right: 0 }}
            >
              {renderRow(item, index)}
            </div>
          );
        })}
      </div>
    </div>
  );
}
