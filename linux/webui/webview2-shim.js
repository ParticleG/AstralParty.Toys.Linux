// Native Linux transport, loaded before every shared WebUI script.
// The shared files talk to `window.chrome.webview` (WebView2). This provides the subset they use,
// postMessage plus 'message' events, on top of the Tauri command, so they run unmodified.
(() => {
  'use strict';
  const invoke = window.__TAURI__.core.invoke;
  const channel = new EventTarget();
  let queue = Promise.resolve();
  let refresh = null;

  const deliver = events => {
    for (const data of events) channel.dispatchEvent(new MessageEvent('message', { data }));
  };

  // One request at a time, so replies are delivered in order. A 'ready' request made while another
  // is pending shares it: the shared WebUI and the overview both ask for the initial state.
  function request(message) {
    if (message?.type === 'ready' && refresh) return refresh;
    const reply = queue.then(async () => deliver(await invoke('host_message', { message })));
    queue = reply.catch(() => {});
    if (message?.type === 'ready') {
      refresh = reply;
      queue.then(() => { if (refresh === reply) refresh = null; });
    }
    return reply;
  }

  window.chrome = Object.assign(window.chrome || {}, {
    webview: {
      postMessage(message) {
        request(message).catch(error => deliver([{ type: 'error', message: String(error?.message ?? error) }]));
      },
      addEventListener: channel.addEventListener.bind(channel),
      removeEventListener: channel.removeEventListener.bind(channel),
    },
  });

  window.toysLinux = {
    request,
    onMessage: listener => channel.addEventListener('message', event => listener(event.data)),
  };
  document.documentElement.dataset.host = 'linux';
})();
