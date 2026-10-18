import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { render } from '@testing-library/react';
import type { ReactElement } from 'react';
import { HashRouter } from 'react-router';

/** Renders with the same providers as main.tsx, using a fresh query cache. */
export function renderWithProviders(ui: ReactElement) {
  const client = new QueryClient({
    defaultOptions: {
      queries: { retry: false, staleTime: Infinity },
      mutations: { retry: false },
    },
  });
  return render(
    <QueryClientProvider client={client}>
      <HashRouter>{ui}</HashRouter>
    </QueryClientProvider>,
  );
}
