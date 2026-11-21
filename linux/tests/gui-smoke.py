#!/usr/bin/env python3
"""Open the packaged app against a disposable Steam library and check the native overview.

Usage: python3 linux/tests/gui-smoke.py /path/to/AstralParty.Toys-linux-x64/astralparty-toys-linux

Needs WebKitWebDriver and a display (run under `xvfb-run -a` on CI). HOME and the XDG
directories point at a temporary fixture, so the real Steam installation is never read, and a
stub `xdg-open` records the launch request instead of passing it to Steam.
"""
import json
import os
import shlex
import socket
import subprocess
import sys
import tempfile
import time
import urllib.error
import urllib.request
from pathlib import Path

EXPECTED = ["国服", "CesiumLoader 未安装", "从 Steam 启动", "安装加载器…", "支持的加载器"]
LAUNCHED = "启动请求已交给 Steam"
STEAM_URI = "steam://rungameid/2622000"
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
    stub = home / "bin/xdg-open"
    stub.parent.mkdir()
    stub.write_text(f'#!/bin/sh\nprintf "%s\\n" "$*" >> {shlex.quote(str(home / "xdg-open.log"))}\n')
    stub.chmod(0o755)


def wait_for(base, session, done, seconds=60):
    state = {}
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        state = call(base, "POST", f"/session/{session}/execute/sync", {"script": OVERVIEW_STATE, "args": []})
        if state["error"] or done(state):
            break
        time.sleep(0.5)
    return state


def main():
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    app = Path(sys.argv[1]).resolve(strict=True)
    with tempfile.TemporaryDirectory(prefix="aspatoys-gui-") as temp:
        home = Path(temp)
        make_fixture(home)
        env = {**os.environ, "HOME": str(home), "XDG_DATA_HOME": str(home / ".local/share"),
               "XDG_CONFIG_HOME": str(home / ".config"), "XDG_CACHE_HOME": str(home / ".cache"),
               "PATH": f"{home / 'bin'}{os.pathsep}{os.environ['PATH']}", "TAURI_WEBVIEW_AUTOMATION": "true"}
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
            state = wait_for(base, session, lambda s: not s["pending"] and all(word in s["text"] for word in EXPECTED))
            missing = [word for word in EXPECTED if word not in state.get("text", "")]
            if state.get("error") or missing or not state.get("sharedHidden"):
                sys.exit(f"Native overview check failed: missing={missing} state={json.dumps(state, ensure_ascii=False)}")
            print("Native overview rendered against the disposable Steam library.")

            button = call(base, "POST", f"/session/{session}/element",
                          {"using": "css selector", "value": '#linuxHome [data-v2-linux="launch"]'})
            call(base, "POST", f"/session/{session}/element/{next(iter(button.values()))}/click", {})
            state = wait_for(base, session, lambda s: LAUNCHED in s["text"], seconds=20)
            log = home / "xdg-open.log"
            requests = log.read_text().split() if log.exists() else []
            if state.get("error") or LAUNCHED not in state.get("text", "") or requests != [STEAM_URI]:
                sys.exit(f"Launch check failed: xdg-open={requests} state={json.dumps(state, ensure_ascii=False)}")
            print(f"Launch button passed {STEAM_URI} to xdg-open.")
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
