'use client';

import * as React from 'react';
import { Moon, Sun, Contrast } from 'lucide-react';
import { Button } from '../primitives/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '../primitives/dropdown-menu';
import { useTheme } from './use-theme';

export function ModeToggle(): React.ReactElement {
  const { setTheme, resolvedTheme } = useTheme();

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon" aria-label="Toggle theme" type="button">
          {resolvedTheme === 'light' ? (
            <Sun className="h-4 w-4" aria-hidden="true" />
          ) : resolvedTheme === 'high_contrast' ? (
            <Contrast className="h-4 w-4" aria-hidden="true" />
          ) : (
            <Moon className="h-4 w-4" aria-hidden="true" />
          )}
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuItem onClick={() => setTheme('light')}>Light</DropdownMenuItem>
        <DropdownMenuItem onClick={() => setTheme('dark')}>Dark</DropdownMenuItem>
        <DropdownMenuItem onClick={() => setTheme('high_contrast')}>High contrast</DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
