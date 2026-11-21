#!/usr/bin/env python3
"""Open the packaged app against a disposable Steam library and check the native overview.

Usage: python3 linux/tests/gui-smoke.py /path/to/AstralParty.Toys-linux-x64/astralparty-toys-linux

Needs WebKitWebDriver and a display (run under `xvfb-run -a` on CI). HOME and the XDG
directories point at a temporary fixture, so the real Steam installation is never read.
"""
import json
import os
import socket
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request
from pathlib import Path

EXPECTED = ["国服", "CesiumLoader 未安装", "从 Steam 启动", "安装加载器…", "支持的加载器"]
OVERVIEW_STATE = """
const text = document.querySelector('#linuxHome')?.innerText || '';
const hidden = sel => { const el = document.querySelector(sel); return !el || getComputedStyle(el).display === 'none'; };
return {
  text,
  pending: !!document.querySelector('#linuxHome .v2-notice[role=status]'),
  error: document.querySelector('#linuxHome .v2-notice.danger')?.innerText || '',
  sharedHidden: hidden('.global-header') && hidden('#v2Home'),
};
"""


def call(base, method, path, body=None):
    data = None if body is None else json.dumps(body).encode()
    request = urllib.request.Request(base + path, data=data, method=method,
                                     headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(request, timeout=60) as response:
        return json.load(response)["value"]


def free_port():
    with socket.socket() as sock:
        sock.bind(("127.0.0.1", 0))
        return sock.getsockname()[1]


def make_fixture(home):
    steamapps = home / ".local/share/Steam/steamapps"
    game = steamapps / "common/Astral Party/8vJXn6CN"
    game.mkdir(parents=True)
    (game / "AstralParty_CN.exe").touch()
    (steamapps / "appmanifest_2622000.acf").write_text(
        'AppState { appid 2622000 installdir "Astral Party" StateFlags 4 }\n')


def main():
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    app = Path(sys.argv[1]).resolve(strict=True)
    with tempfile.TemporaryDirectory(prefix="aspatoys-gui-") as temp:
        home = Path(temp)
        make_fixture(home)
        env = {**os.environ, "HOME": str(home), "XDG_DATA_HOME": str(home / ".local/share"),
               "XDG_CONFIG_HOME": str(home / ".config"), "XDG_CACHE_HOME": str(home / ".cache"),
               "TAURI_WEBVIEW_AUTOMATION": "true"}
        port = free_port()
        base = f"http://127.0.0.1:{port}"
        driver = subprocess.Popen(["WebKitWebDriver", f"--port={port}"], env=env,
                                  stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        session = None
        try:
            for _ in range(100):
                try:
                    if call(base, "GET", "/status").get("ready"):
                        break
                except (urllib.error.URLError, ConnectionError):
                    time.sleep(0.1)
            options = {"binary": str(app), "args": []}
            session = call(base, "POST", "/session",
                           {"capabilities": {"alwaysMatch": {"webkitgtk:browserOptions": options}}})["sessionId"]
            state = {}
            deadline = time.monotonic() + 60
            while time.monotonic() < deadline:
                state = call(base, "POST", f"/session/{session}/execute/sync",
                             {"script": OVERVIEW_STATE, "args": []})
                if state["error"] or (not state["pending"] and all(word in state["text"] for word in EXPECTED)):
                    break
                time.sleep(0.5)
            missing = [word for word in EXPECTED if word not in state.get("text", "")]
            if state.get("error") or missing or not state.get("sharedHidden"):
                sys.exit(f"Native overview check failed: missing={missing} state={json.dumps(state, ensure_ascii=False)}")
            print("Native overview rendered against the disposable Steam library.")
        finally:
            if session:
                try:
                    call(base, "DELETE", f"/session/{session}")
                except (urllib.error.URLError, ConnectionError):
                    pass
            driver.terminate()
            driver.wait(timeout=10)


if __name__ == "__main__":
    main()
