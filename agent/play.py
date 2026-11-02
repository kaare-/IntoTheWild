#!/usr/bin/env python3
"""One external player. Scripted by default; a model is optional.

The world does not wait. If this process is slow, the window closes and the
next command is late: it joins the following window instead of being dropped.
"""

from __future__ import annotations

import argparse
import time

from policy import Memory, choose
from villmarks import Client, ValleyError


def main() -> None:
    parser = argparse.ArgumentParser(description="Play Villmarks through the player API")
    parser.add_argument("--base", default="http://127.0.0.1:18431")
    parser.add_argument("--name", default="Linde")
    parser.add_argument("--policy", choices=("scripted", "model"), default="scripted")
    parser.add_argument("--turns", type=int, default=40)
    parser.add_argument("--manual-advance", action="store_true")
    args = parser.parse_args()
    if args.policy == "model" and not __import__("os").environ.get("VILLMARKS_MODEL_URL"):
        raise SystemExit("Set VILLMARKS_MODEL_URL, VILLMARKS_MODEL_NAME, and VILLMARKS_MODEL_KEY")

    client = Client(args.base)
    joined = client.join(args.name)
    print(f"joined as {joined['name']} token={joined['token']}")
    memory = Memory()
    last_time = None
    for _ in range(args.turns):
        if args.manual_advance:
            client.advance(1)
        else:
            wait_for_next_window(client, last_time)
        obs = client.observe()
        last_time = obs["sim_time"]
        command = choose(obs, memory)
        print(f"t{obs['sim_time']} {'rain' if obs['raining'] else 'dry'} -> {command or '(wait)'}")
        if command:
            try:
                ack = client.intent(command)
            except ValleyError as err:
                print(f"  refused: {err}")
                continue
            print(f"  {ack['message']}")
    final = client.observe()
    print(final["text"])
    print(final["map"])


def wait_for_next_window(client: Client, previous: int | None) -> None:
    started = time.time()
    while time.time() - started < 120:
        clock = client.clock()
        if previous is None or clock["sim_time"] != previous:
            if previous is None:
                # Act on the open window, then wait out its close next loop.
                return
            return
        time.sleep(0.4)


if __name__ == "__main__":
    main()
