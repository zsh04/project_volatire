import type { Metadata } from 'next';
import { Inter, JetBrains_Mono } from 'next/font/google';
import './globals.css';

import { LiveTelemetryProvider } from '@/lib/grpc/LiveTelemetryProvider';

const inter = Inter({ subsets: ['latin'], variable: '--font-inter' });
const mono = JetBrains_Mono({ subsets: ['latin'], variable: '--font-mono' });

export const metadata: Metadata = {
  title: 'Voltaire Command Deck',
  description: 'High-frequency trading HUD for Project Voltaire',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body className={`${inter.variable} ${mono.variable} font-sans bg-black text-white selection:bg-emerald-500/30 selection:text-emerald-200`} suppressHydrationWarning>
        <LiveTelemetryProvider>
          {children}
        </LiveTelemetryProvider>
      </body>
    </html>
  );
}
