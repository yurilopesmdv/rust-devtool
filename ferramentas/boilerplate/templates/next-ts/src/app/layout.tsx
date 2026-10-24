import React, { type ReactNode } from 'react';
export const metadata = { title: '{{PROJECT_NAME}}' };
export default function Layout({ children }: { children: ReactNode }) {
  return (
    <html lang='pt-BR'>
      <body>{children}</body>
    </html>
  );
}
