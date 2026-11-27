#!/usr/bin/env python3
"""Join, walk, stop the server, start it again, resume from the token file.

Proves Slice 1: the same Bearer token (saved under data/tokens/) reattaches
to the same body after a restart. Uses the player HTTP API only.
"""

from __future__ import annotations

import os
import signal
import subprocess
import sys
import time
import urllib.request
from pathlib import Path

from villmarks import Client, token_path_for

ROOT = Path(__file__).resolve().parents[1]


def wait_health(base: str) -> None:
    deadline = time.time() + 30
    while time.time() < deadline:
        try:
            with urllib.request.urlopen(base + "/health", timeout=1) as response:
                if response.status == 200:
                    return
        except Exception:
            time.sleep(0.2)
    raise SystemExit(f"server did not answer {base}/health")


def start(binary: str, data: str, port: int) -> subprocess.Popen[bytes]:
    return subprocess.Popen(
        [
            binary,
            "--manual",
            "--port",
            str(port),
            "--data",
            data,
            "--snapshot-every",
            "2",
            "--static",
            str(ROOT / "web" / "dist"),
        ],
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )


def stop(proc: subprocess.Popen[bytes]) -> None:
    proc.send_signal(signal.SIGINT)
    try:
        proc.wait(timeout=10)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait(timeout=5)


def main() -> None:
    binary = sys.argv[1] if len(sys.argv) > 1 else str(ROOT / "target" / "debug" / "villmarks-server")
    data = "/tmp/villmarks-reconnect.sqlite"
    for suffix in ("", "-wal", "-shm"):
        try:
            os.remove(data + suffix)
        except FileNotFoundError:
            pass
    token_file = token_path_for("Sera")
    try:
        token_file.unlink()
    except FileNotFoundError:
        pass

    port = 18461
    base = f"http://127.0.0.1:{port}"
    proc = start(binary, data, port)
    before_xy = (0, 0)
    before_time = 0
    saved_token = ""
    try:
        wait_health(base)
        client = Client(base)
        joined = client.connect("Sera", token_file=token_file, fresh=True)
        if joined["resumed"]:
            raise SystemExit("expected a fresh join")
        saved_token = joined["token"]
        print(f"joined {joined['name']} token_file={token_file}")
        client.intent("n")
        client.advance(1)
        client.intent("w")
        client.advance(1)
        before = client.observe()
        before_xy = (before["self_info"]["x"], before["self_info"]["y"])
        before_time = before["sim_time"]
        print(f"before stop at ({before_xy[0]},{before_xy[1]}) t{before_time}")
        print(before["text"].splitlines()[0])
    finally:
        stop(proc)

    # New client process, same token file, same sqlite.
    proc = start(binary, data, port)
    try:
        wait_health(base)
        client = Client(base)
        again = client.connect("Sera", token_file=token_file)
        if not again["resumed"]:
            raise SystemExit("did not resume from the token file")
        if again["token"] != saved_token:
            raise SystemExit("token file did not match the first join")
        after = client.observe()
        print("--- after reconnect ---")
        print(after["text"].splitlines()[0])
        print(f"time {after['sim_time']} name={after['self_info']['name']}")
        if after["sim_time"] != before_time:
            raise SystemExit(f"clock changed: {before_time} -> {after['sim_time']}")
        if (after["self_info"]["x"], after["self_info"]["y"]) != before_xy:
            raise SystemExit("position changed across reconnect")
        if after["self_info"]["name"] != "Sera":
            raise SystemExit("name changed across reconnect")
        # Still the same body: another join with the same name must fail.
        clash = Client(base)
        try:
            clash.join("Sera")
            raise SystemExit("same name was allowed a second body")
        except Exception as err:
            print(f"name still taken: {err}")
        print("PASS reconnect")
    finally:
        stop(proc)


if __name__ == "__main__":
    main()
