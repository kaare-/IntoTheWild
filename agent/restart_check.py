#!/usr/bin/env python3
"""Stop and start the server while two people hold the log.

Uses the player HTTP API. SIGINT asks the process to snapshot and exit.
"""

from __future__ import annotations

import os
import signal
import subprocess
import sys
import time
import urllib.request
from pathlib import Path

from villmarks import Client

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
    data = "/tmp/villmarks-restart.sqlite"
    for suffix in ("", "-wal", "-shm"):
        try:
            os.remove(data + suffix)
        except FileNotFoundError:
            pass
    port = 18451
    base = f"http://127.0.0.1:{port}"
    proc = start(binary, data, port)
    sera_token = ""
    otto_token = ""
    grab = "restart-grab-sera"
    before_time = 0
    before_xy = (0, 0)
    try:
        wait_health(base)
        sera = Client(base)
        otto = Client(base)
        sera.join("Sera")
        otto.join("Otto")
        sera_token = sera.token or ""
        otto_token = otto.token or ""
        sera.intent("n")
        otto.intent("n")
        sera.advance(1)
        sera.intent("w")
        sera.advance(1)
        print("grab", sera.intent("grab", grab)["message"])
        print("grab", otto.intent("grab")["message"])
        sera.advance(1)
        while not sera.observe()["raining"]:
            sera.advance(1)
        # Seal the drag so it is in progress, and stop before it completes.
        sera.intent("drag e")
        otto.intent("drag e")
        sera.advance(1)
        before = sera.observe()
        print(before["text"])
        print(before["work"])
        if not before["self_info"]["gripping"] or not otto.observe()["self_info"]["gripping"]:
            raise SystemExit("both players were not gripping before the stop")
        if not before["work"]:
            raise SystemExit("the drag was not still in progress")
        before_time = before["sim_time"]
        before_xy = (before["self_info"]["x"], before["self_info"]["y"])
    finally:
        stop(proc)

    proc = start(binary, data, port)
    try:
        wait_health(base)
        sera = Client(base, sera_token)
        otto = Client(base, otto_token)
        after = sera.observe()
        otto_after = otto.observe()
        print("--- after restart ---")
        print(after["text"])
        print(f"time {after['sim_time']} raining {after['raining']}")
        if after["sim_time"] != before_time:
            raise SystemExit(f"clock changed across restart: {before_time} -> {after['sim_time']}")
        if not after["self_info"]["gripping"] or not otto_after["self_info"]["gripping"]:
            raise SystemExit("grips did not survive the restart")
        if (after["self_info"]["x"], after["self_info"]["y"]) != before_xy:
            raise SystemExit("position changed across restart")
        if not after["raining"]:
            raise SystemExit("rain did not survive the restart")
        if not after["work"] or after["work"]["finishes_at"] != before["work"]["finishes_at"]:
            raise SystemExit(f"the drag schedule changed: {after.get('work')}")
        again = sera.intent("grab", grab)
        print(f"resubmit duplicate={again['duplicate']} message={again['message']}")
        if not again["duplicate"]:
            raise SystemExit("the same command id ran again")
        sera.advance(1)
        continued = sera.observe()
        print(f"continued at ({continued['self_info']['x']},{continued['self_info']['y']}) work={continued['work']}")
        if continued["self_info"]["x"] != before_xy[0] + 1 or continued["self_info"]["y"] != before_xy[1]:
            raise SystemExit("the drag did not finish one cell east after the restart")
        if continued["work"] is not None:
            raise SystemExit("the drag was still running after its tick")
        if not continued["self_info"]["gripping"]:
            raise SystemExit("the grip was dropped when the drag finished")
        print("PASS restart")
    finally:
        stop(proc)


if __name__ == "__main__":
    main()
