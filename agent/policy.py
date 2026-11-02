"""Scripted shelter policy, plus an optional external model.

The model is a client. Its latency never enters the valley clock. If the
model is unset, slow, or unusable, the scripted policy chooses.
"""

from __future__ import annotations

import json
import os
import urllib.request

DIRS = {"n": (0, -1), "e": (1, 0), "s": (0, 1), "w": (-1, 0)}


class Memory:
    def __init__(self) -> None:
        self.cells: dict[tuple[int, int], dict] = {}
        self.width = 7
        self.height = 5

    def see(self, obs: dict) -> None:
        rows = [line for line in obs["map"].splitlines() if line.strip()]
        if len(rows) >= 2:
            self.height = len(rows) - 1
            self.width = max(self.width, rows[0].count("  ") )
        for cell in obs["visible_cells"]:
            self.cells[(cell["x"], cell["y"])] = cell


def choose(obs: dict, memory: Memory) -> str | None:
    """Return one command, or None to let the window pass."""
    if os.environ.get("VILLMARKS_MODEL_URL"):
        suggested = ask_model(obs)
        if suggested:
            return suggested
    return scripted(obs, memory)


def scripted(obs: dict, memory: Memory) -> str | None:
    memory.see(obs)
    if obs.get("work"):
        return None
    allowed = set(obs["allowed_actions"])
    if me_thirsty(obs) and "drink" in allowed:
        return "drink"
    me = obs["self_info"]
    here = memory.cells.get((me["x"], me["y"]), {})
    branches = carry(me, "branch")
    bark = carry(me, "bark")

    if me["gripping"]:
        roofs = [
            pos
            for pos, cell in memory.cells.items()
            if cell.get("roof") in {"rock", "bark"} or cell.get("under_roof")
        ]
        target = nearest((me["x"], me["y"]), roofs)
        if target and target != (me["x"], me["y"]):
            return drag_toward(me, target, allowed)
        for direction in ("e", "n", "s", "w"):
            command = f"drag {direction}"
            if command in allowed:
                return command
        return None

    if "lay bark" in allowed and bark and here.get("frame") == "supported":
        return "lay bark"
    if (
        "raise frame" in allowed
        and branches >= 3
        and here.get("branch_pile", 0) > 0
        and not here.get("under_roof")
    ):
        return "raise frame"

    if bark == 0:
        bark_cells = [pos for pos, cell in memory.cells.items() if cell.get("bark_pile", 0) > 0]
        if here.get("bark_pile", 0) > 0 and "gather bark" in allowed:
            return "gather bark"
        step = walk_toward(me, nearest((me["x"], me["y"]), bark_cells), allowed)
        if step:
            return step

    if branches < 3:
        stocks = []
        for pos, cell in memory.cells.items():
            pile = cell.get("branch_pile", 0)
            if pile <= 0:
                continue
            if not cell.get("under_roof") and pile <= 1:
                continue
            stocks.append(pos)
        if here.get("branch_pile", 0) > 0 and "gather branch" in allowed:
            leave_one = not here.get("under_roof") and here.get("branch_pile", 0) <= 1
            if not leave_one:
                return "gather branch"
        step = walk_toward(me, nearest((me["x"], me["y"]), stocks), allowed)
        if step:
            return step

    open_wood = [
        pos
        for pos, cell in memory.cells.items()
        if cell.get("branch_pile", 0) > 0 and not cell.get("under_roof")
    ]
    if branches >= 3 and bark and open_wood:
        step = walk_toward(me, nearest((me["x"], me["y"]), open_wood), allowed)
        if step:
            return step
        if "raise frame" in allowed:
            return "raise frame"

    helpers = []
    for pos, cell in memory.cells.items():
        if cell.get("log") and cell.get("log_gripped_by") and "grab" not in allowed:
            helpers.append(pos)
        if cell.get("log"):
            helpers.append(pos)
    if here.get("log") and "grab" in allowed and not me["gripping"]:
        if here.get("log_gripped_by") or obs["raining"]:
            return "grab"
    if helpers and not branches:
        step = walk_toward(me, nearest((me["x"], me["y"]), helpers), allowed)
        if step:
            return step

    return explore(me, memory, allowed)


def ask_model(obs: dict) -> str | None:
    url = os.environ["VILLMARKS_MODEL_URL"]
    model = os.environ.get("VILLMARKS_MODEL_NAME", "gpt-4o-mini")
    key = os.environ.get("VILLMARKS_MODEL_KEY", "")
    timeout = float(os.environ.get("VILLMARKS_MODEL_TIMEOUT", "8"))
    allowed = obs["allowed_actions"]
    prompt = (
        "You are in a wilderness MUD. Choose exactly one command from the allowed list. "
        "Reply with the command only.\n\n"
        f"Observation:\n{obs['text']}\n\nMap:\n{obs['map']}\n\nAllowed:\n"
        + "\n".join(allowed)
    )
    payload = {
        "model": model,
        "temperature": 0,
        "messages": [
            {"role": "system", "content": "Reply with one allowed command and nothing else."},
            {"role": "user", "content": prompt},
        ],
    }
    request = urllib.request.Request(
        url,
        data=json.dumps(payload).encode(),
        headers={"Content-Type": "application/json", **({"Authorization": f"Bearer {key}"} if key else {})},
    )
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            body = json.loads(response.read().decode())
        text = body["choices"][0]["message"]["content"].strip().splitlines()[0].strip().lower()
    except Exception:
        return None
    if text in {item.lower() for item in allowed}:
        for item in allowed:
            if item.lower() == text:
                return item
    return None


def me_thirsty(obs: dict) -> bool:
    return obs.get("self_info", {}).get("thirst") in {"thirsty", "parched"}


def carry(me: dict, material: str) -> int:
    return sum(item["count"] for item in me["carrying"] if item["material"] == material)


def nearest(origin: tuple[int, int], goals: list[tuple[int, int]]) -> tuple[int, int] | None:
    if not goals:
        return None
    return min(goals, key=lambda pos: max(abs(pos[0] - origin[0]), abs(pos[1] - origin[1])))


def walk_toward(me: dict, target: tuple[int, int] | None, allowed: set[str]) -> str | None:
    if target is None:
        return None
    dx = target[0] - me["x"]
    dy = target[1] - me["y"]
    order = []
    if abs(dx) >= abs(dy):
        if dx > 0:
            order.append("e")
        elif dx < 0:
            order.append("w")
        if dy > 0:
            order.append("s")
        elif dy < 0:
            order.append("n")
    else:
        if dy > 0:
            order.append("s")
        elif dy < 0:
            order.append("n")
        if dx > 0:
            order.append("e")
        elif dx < 0:
            order.append("w")
    for direction in order:
        if direction in allowed:
            return direction
    return None


def drag_toward(me: dict, target: tuple[int, int], allowed: set[str]) -> str | None:
    step = walk_toward(me, target, {"n", "e", "s", "w"})
    if step and f"drag {step}" in allowed:
        return f"drag {step}"
    for direction in ("e", "n", "s", "w"):
        if f"drag {direction}" in allowed:
            return f"drag {direction}"
    return None


def explore(me: dict, memory: Memory, allowed: set[str]) -> str | None:
    unknown = [
        (x, y)
        for y in range(memory.height)
        for x in range(memory.width)
        if (x, y) not in memory.cells
    ]
    step = walk_toward(me, nearest((me["x"], me["y"]), unknown), allowed)
    if step:
        return step
    for direction in ("e", "s", "w", "n"):
        if direction in allowed:
            return direction
    return "look" if "look" in allowed else None
