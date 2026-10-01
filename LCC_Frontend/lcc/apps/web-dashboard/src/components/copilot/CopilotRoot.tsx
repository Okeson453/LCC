'use client';

import * as React from 'react';
import { Sparkles, X } from 'lucide-react';
import { Button, Sheet, SheetContent, SheetHeader, SheetTitle } from '@lcc/ui';
import { useCopilotStore } from '@/lib/stores';
import { CopilotPanel } from './CopilotPanel';

export function CopilotRoot({ fullPage }: { fullPage?: boolean }): React.ReactElement {
  const open = useCopilotStore((s) => s.open);
  const setOpen = useCopilotStore((s) => s.setOpen);

  return (
    <>
      {!open ? (
        <Button
          size="icon"
          onClick={() => setOpen(true)}
          type="button"
          aria-label="Open Copilot"
          className="fixed bottom-4 right-4 z-40 h-12 w-12 rounded-full shadow-lg"
        >
          <Sparkles className="h-5 w-5" aria-hidden="true" />
        </Button>
      ) : null}
      <Sheet open={open} onOpenChange={setOpen}>
        <SheetContent side="right" className={`flex w-full flex-col p-0 ${fullPage ? 'sm:max-w-none h-screen' : 'sm:max-w-md'}`}>
          <SheetHeader className="flex flex-row items-center justify-between border-b p-4">
            <SheetTitle>Copilot</SheetTitle>
            <Button
              size="icon"
              variant="ghost"
              onClick={() => setOpen(false)}
              type="button"
              aria-label="Close Copilot"
            >
              <X className="h-4 w-4" aria-hidden="true" />
            </Button>
          </SheetHeader>
          <CopilotPanel />
        </SheetContent>
      </Sheet>
    </>
  );
}
