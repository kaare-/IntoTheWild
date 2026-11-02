#!/usr/bin/env python3
"""Same seed, opposite HTTP receive order, same log movement."""

from __future__ import annotations

import os
import signal
import subprocess
import time
import urllib.request
from pathlib import Path

from villmarks import Client

ROOT = Path(__file__).resolve().parents[1]


def start(port: int, data: str) -> subprocess.Popen[bytes]:
    for suffix in ("", "-wal", "-shm"):
        try:
            os.remove(data + suffix)
        except FileNotFoundError:
            pass
    return subprocess.Popen(
        [
            str(ROOT / "target" / "debug" / "villmarks-server"),
            "--manual",
            "--port",
            str(port),
            "--data",
            data,
            "--seed",
            "7",
            "--static",
            str(ROOT / "web" / "dist"),
        ],
        cwd=ROOT,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.STDOUT,
    )


def wait_health(port: int) -> None:
    base = f"http://127.0.0.1:{port}/health"
    for _ in range(50):
        try:
            with urllib.request.urlopen(base, timeout=1) as response:
                if response.status == 200:
                    return
        except Exception:
            time.sleep(0.1)
    raise SystemExit(f"no server on {port}")


def play(port: int, sera_first: bool) -> tuple[int, int]:
    base = f"http://127.0.0.1:{port}"
    sera = Client(base)
    otto = Client(base)
    sera.join("Sera")
    otto.join("Otto")
    sera.intent("n")
    otto.intent("n")
    sera.advance(1)
    sera.intent("w")
    sera.advance(1)
    first, second = (sera, otto) if sera_first else (otto, sera)
    first.intent("grab")
    second.intent("grab")
    sera.advance(1)
    first.intent("drag e")
    second.intent("drag e")
    sera.advance(1)
    sera.advance(1)
    obs = sera.observe()
    return obs["self_info"]["x"], obs["self_info"]["y"]


def main() -> None:
    left = start(18461, "/tmp/villmarks-order-a.sqlite")
    right = start(18462, "/tmp/villmarks-order-b.sqlite")
    try:
        wait_health(18461)
        wait_health(18462)
        a = play(18461, True)
        b = play(18462, False)
        print(f"sera-first {a} otto-first {b}")
        if a != b or a != (4, 2):
            raise SystemExit(f"receive order changed the drag: {a} vs {b}")
        print("PASS order")
    finally:
        for proc in (left, right):
            proc.send_signal(signal.SIGINT)
            try:
                proc.wait(timeout=8)
            except subprocess.TimeoutExpired:
                proc.kill()


if __name__ == "__main__":
    main()
