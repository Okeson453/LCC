import * as React from 'react';
import { redirect } from 'next/navigation';
import { auth } from '@/lib/auth/server';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@lcc/ui';

export default async function AdminLayout({ children }: { children: React.ReactNode }): Promise<React.ReactElement> {
  const session = await auth();
  if (!session || session.user.role !== 'admin') {
    return (
      <main className="flex min-h-[60vh] items-center justify-center p-6">
        <Card className="max-w-md">
          <CardHeader>
            <CardTitle>Forbidden</CardTitle>
            <CardDescription>This area requires admin role.</CardDescription>
          </CardHeader>
          <CardContent>
            <a className="text-primary underline" href="/restricted">Restricted state</a>
          </CardContent>
        </Card>
      </main>
    );
  }
  return <>{children}</>;
}
