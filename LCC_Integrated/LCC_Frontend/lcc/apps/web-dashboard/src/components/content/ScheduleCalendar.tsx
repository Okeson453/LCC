'use client';

import * as React from 'react';
import { Calendar } from '@lcc/ui';

export interface ScheduleCalendarProps {
  value: string;
  onChange: (iso: string) => void;
  minDate?: string;
}

export function ScheduleCalendar({ value, onChange, minDate }: ScheduleCalendarProps): React.ReactElement {
  return (
    <Calendar
      mode="single"
      selected={value ? new Date(value) : undefined}
      onSelect={(d) => onChange(d ? d.toISOString() : '')}
      disabled={(d) => (minDate ? d < new Date(minDate) : false)}
    />
  );
}
