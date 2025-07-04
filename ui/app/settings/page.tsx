'use client';

import * as React from 'react';
import { NostrWalletSettings } from '@/components/settings/nostr-wallet-settings';
import { SiteHeader } from '@/components/site-header';
import { AppSidebar } from '@/components/app-sidebar';
import { SidebarInset, SidebarProvider } from '@/components/ui/sidebar';
import { Toaster } from 'sonner';

export default function SettingsPage() {
  return (
    <SidebarProvider>
      <AppSidebar variant='inset' />
      <SidebarInset>
        <SiteHeader />
        <div className='flex flex-1 flex-col'>
          <div className='@container/main flex flex-1 flex-col gap-4 p-4 md:gap-8 md:p-8'>
            <div className='flex items-center'>
              <h1 className='text-2xl font-bold tracking-tight'>Settings</h1>
            </div>
            <div className='w-full'>
              <NostrWalletSettings />
            </div>
          </div>
        </div>
      </SidebarInset>
      <Toaster />
    </SidebarProvider>
  );
}
