#!/usr/bin/env node
// Inspect the running desktop app through the Chrome DevTools Protocol.
//
// Start the app with remote debugging enabled (Windows/WebView2):
//   set WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222
// then run:
//   node scripts/inspect-webview.mjs [--port 9222] [--screenshot out.png] [--eval "js expr"]
//
// Prints the page title and visible text; optionally saves a screenshot or
// evaluates an expression. Used to verify milestones against the real app.
import { writeFileSync } from 'node:fs';

const args = process.argv.slice(2);
const opt = (name) => {
  const i = args.indexOf(name);
  return i >= 0 ? args[i + 1] : undefined;
};
const port = Number(opt('--port') ?? 9222);
const screenshot = opt('--screenshot');
const expression = opt('--eval');

async function findPage(retries = 30) {
  for (let i = 0; i < retries; i++) {
    try {
      const res = await fetch(`http://127.0.0.1:${port}/json/list`);
      const targets = await res.json();
      const page = targets.find((t) => t.type === 'page');
      if (page) return page;
    } catch {
      // App not listening yet.
    }
    await new Promise((r) => setTimeout(r, 1000));
  }
  throw new Error(`No debuggable page on port ${port} after ${retries}s`);
}

function connect(url) {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(url);
    let nextId = 1;
    const pending = new Map();
    ws.addEventListener('message', (event) => {
      const msg = JSON.parse(String(event.data));
      if (msg.id && pending.has(msg.id)) {
        const { ok, fail } = pending.get(msg.id);
        pending.delete(msg.id);
        if (msg.error) fail(new Error(msg.error.message));
        else ok(msg.result);
      }
    });
    ws.addEventListener('open', () =>
      resolve({
        send(method, params = {}) {
          const id = nextId++;
          ws.send(JSON.stringify({ id, method, params }));
          return new Promise((ok, fail) => pending.set(id, { ok, fail }));
        },
        close: () => ws.close(),
      }),
    );
    ws.addEventListener('error', () => reject(new Error(`Cannot connect to ${url}`)));
  });
}

const page = await findPage();
const cdp = await connect(page.webSocketDebuggerUrl);
try {
  const evaluate = async (expr) => {
    const { result, exceptionDetails } = await cdp.send('Runtime.evaluate', {
      expression: expr,
      awaitPromise: true,
      returnByValue: true,
    });
    if (exceptionDetails) throw new Error(exceptionDetails.text);
    return result.value;
  };

  // Give the UI a moment to finish its first IPC round-trip.
  await new Promise((r) => setTimeout(r, 1500));
  console.log(`URL:   ${page.url}`);
  console.log(`Title: ${await evaluate('document.title')}`);
  console.log('--- visible text ---');
  console.log(await evaluate('document.body.innerText'));

  if (expression) {
    console.log('--- eval ---');
    console.log(JSON.stringify(await evaluate(expression), null, 2));
  }
  if (screenshot) {
    const { data } = await cdp.send('Page.captureScreenshot', { format: 'png' });
    writeFileSync(screenshot, Buffer.from(data, 'base64'));
    console.log(`Screenshot saved to ${screenshot}`);
  }
} finally {
  cdp.close();
}
