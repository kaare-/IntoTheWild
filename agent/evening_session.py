#!/usr/bin/env python3
"""Play the rain evening through the same HTTP API the browser uses.

Expects a server started with --manual. Each turn submits every player's
intent for the open window, then advances one tick.
"""

from __future__ import annotations

import sys

from villmarks import Client, ValleyError


def main() -> None:
    base = sys.argv[1] if len(sys.argv) > 1 else "http://127.0.0.1:18441"
    sera = Client(base)
    otto = Client(base)
    linde = Client(base)
    print(sera.join("Sera")["name"], otto.join("Otto")["name"], linde.join("Linde")["name"])

    # Index is the window. None means that body waits.
    # Sera and Otto share the log. Linde roofs a frame on open ground
    # with wood and bark taken from under the rock, before tick 16.
    sera_plan = {
        0: "n",
        1: "w",
        2: "grab",
        3: "drag e",
        4: "interrupt",
        5: "drag e",
        7: "examine log",
        8: "examine wood",
    }
    otto_plan = {
        0: "n",
        2: "grab",
        3: "drag e",
        5: "drag e",
        7: "say The log can wait. The sky will not.",
    }
    linde_plan = {
        0: "n",
        1: "n",
        2: "e",
        3: "gather branch",
        4: "gather branch",
        5: "gather branch",
        6: "gather bark",
        7: "w",
        8: "raise frame",
        12: "lay bark",
        14: "examine frame",
        15: "examine wood",
    }

    grab_id = "sera-grab-once"
    notes: list[str] = []
    for tick in range(22):
        for client, plan in ((sera, sera_plan), (otto, otto_plan), (linde, linde_plan)):
            line = plan.get(tick)
            if not line:
                continue
            command_id = grab_id if line == "grab" and client is sera else None
            try:
                ack = client.intent(line, command_id)
            except ValleyError as err:
                notes.append(f"t{tick} REFUSED {line}: {err}")
                print(notes[-1])
                continue
            notes.append(f"t{tick} {ack['message'].splitlines()[0]}")
            print(notes[-1])
        if tick == 0:
            sera.intent("say The wood under the rock is still dry.")
        advanced = sera.advance(1)
        obs = sera.observe()
        print(
            f"  now t{advanced['sim_time']} rain={obs['raining']} "
            f"sera=({obs['self_info']['x']},{obs['self_info']['y']}) "
            f"grip={obs['self_info']['gripping']}"
        )

    again = sera.intent("grab", grab_id)
    print(f"duplicate grab duplicate={again['duplicate']} message={again['message']}")

    # Rain has been falling. Read the wood from someone standing by the frame
    # and from Sera, who can see the open pile and the rock.
    linde_wood = linde.intent("examine wood")["message"]
    linde_frame = linde.intent("examine frame")["message"]
    sera_wood = sera.intent("examine wood")["message"]
    far = Client(base)
    far.join("Nils")
    # Nils enters on the south trail. Four steps west puts the wood outside his sight.
    for step in ("w", "w", "w", "w"):
        far.intent(step)
        far.advance(1)
    far_obs = far.observe()
    far_wood = far.intent("examine wood")["message"]

    print("--- linde frame ---")
    print(linde_frame)
    print("--- linde wood ---")
    print(linde_wood)
    print("--- sera wood ---")
    print(sera_wood)
    print("--- far map ---")
    print(far_obs["map"])
    print("--- far wood ---")
    print(far_wood)
    print("--- sera map ---")
    print(sera.observe()["map"])

    problems = []
    if not again["duplicate"]:
        problems.append("resubmitted grab was not marked duplicate")
    if "supported at both ends" not in linde_frame:
        problems.append("frame was not supported at both ends")
    if "bark" not in linde_frame.lower() and "roof" not in linde_frame.lower():
        problems.append("bark roof not described")
    if "soaked" not in sera_wood and "soaked" not in linde_wood:
        problems.append("no soaked wood was visible after rain")
    if "dry" not in sera_wood and "dry" not in linde_frame:
        problems.append("no dry wood remained visible")
    if far_wood != "You see no wood from here.":
        problems.append(f"distant character saw wood: {far_wood}")
    if "w" in far_obs["map"] or "W" in far_obs["map"]:
        problems.append("distant map showed wood glyphs")
    if problems:
        print("FAIL")
        for item in problems:
            print(" -", item)
        sys.exit(1)
    print("PASS evening session")


if __name__ == "__main__":
    main()
