import '@fontsource-variable/inter';
import '@fontsource-variable/source-serif-4';
import './i18n';
import './styles/index.css';

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { HashRouter } from 'react-router';

import { App } from './app/App';
import { commands } from './ipc/commands';

// Forward uncaught UI errors to the app log so they show up in diagnostics.
function report(message: string) {
  commands.logFrontend('error', message).catch(() => undefined);
}
window.addEventListener('error', (e) => {
  report(`${e.message} at ${e.filename}:${String(e.lineno)}`);
});
window.addEventListener('unhandledrejection', (e: PromiseRejectionEvent) => {
  report(`unhandled rejection: ${JSON.stringify(e.reason) || String(e.reason)}`);
});

const queryClient = new QueryClient({
  defaultOptions: {
    // IPC calls are local; retrying hides real errors.
    queries: { retry: false, refetchOnWindowFocus: false, staleTime: Infinity },
    mutations: { retry: false },
  },
});

const root = document.getElementById('root');
if (!root) {
  throw new Error('Root element #root is missing from index.html');
}

createRoot(root).render(
  <StrictMode>
    <QueryClientProvider client={queryClient}>
      <HashRouter>
        <App />
      </HashRouter>
    </QueryClientProvider>
  </StrictMode>,
);
