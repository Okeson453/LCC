/**
 * Common column def helpers for DataTable.
 */

import { type ColumnDef } from '@tanstack/react-table';

export function textColumn<T>(
  accessor: keyof T & string,
  header: string,
): ColumnDef<T, unknown> {
  return {
    accessorKey: accessor,
    header,
    cell: (info) => {
      const v = info.getValue();
      return v == null ? '—' : String(v);
    },
  };
}

export function dateColumn<T>(
  accessor: keyof T & string,
  header: string,
): ColumnDef<T, unknown> {
  return {
    accessorKey: accessor,
    header,
    cell: (info) => {
      const v = info.getValue();
      if (v == null) return '—';
      const d = new Date(v as string | number);
      return d.toLocaleDateString();
    },
  };
}

export function numericColumn<T>(
  accessor: keyof T & string,
  header: string,
): ColumnDef<T, unknown> {
  return {
    accessorKey: accessor,
    header,
    cell: (info) => {
      const v = info.getValue();
      if (typeof v !== 'number') return '—';
      return v.toLocaleString();
    },
  };
}
