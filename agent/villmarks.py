"""Player API client. Humans and this process use the same routes."""

from __future__ import annotations

import json
import re
import urllib.error
import urllib.request
import uuid
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TOKEN_DIR = ROOT / "data" / "tokens"


class ValleyError(RuntimeError):
    pass


def token_path_for(name: str, directory: Path | None = None) -> Path:
    """Default on-disk path for a body's Bearer token."""
    safe = re.sub(r"[^A-Za-z0-9._-]+", "_", name.strip()) or "player"
    return (directory or TOKEN_DIR) / f"{safe}.token"


def load_token(path: Path) -> str | None:
    try:
        text = path.read_text(encoding="utf-8").strip()
    except FileNotFoundError:
        return None
    except OSError as err:
        raise ValleyError(f"could not read token file {path}: {err}") from err
    if not text:
        return None
    # One line of hex, or a tiny JSON blob with "token".
    if text.startswith("{"):
        try:
            value = json.loads(text).get("token")
        except json.JSONDecodeError as err:
            raise ValleyError(f"token file {path} is not valid JSON") from err
        return value.strip() if isinstance(value, str) and value.strip() else None
    return text.splitlines()[0].strip() or None


def save_token(path: Path, token: str, name: str | None = None) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    payload = token.strip() + "\n"
    if name:
        payload = json.dumps({"name": name, "token": token.strip()}, indent=2) + "\n"
    path.write_text(payload, encoding="utf-8")


class Client:
    def __init__(self, base: str, token: str | None = None) -> None:
        self.base = base.rstrip("/")
        self.token = token

    def join(self, name: str) -> dict:
        body = self._request("POST", "/v1/join", {"name": name}, auth=False)
        self.token = body["token"]
        return body

    def resume(self, token: str) -> dict:
        """Attach to an existing body. Raises ValleyError if the token is unknown."""
        self.token = token.strip()
        return self.observe()

    def connect(
        self,
        name: str,
        *,
        token: str | None = None,
        token_file: Path | None = None,
        fresh: bool = False,
    ) -> dict:
        """Resume a saved token when possible; otherwise join and save.

        Returns a dict with at least name, token, and resumed (bool). When a
        new body is created, actor_id is included from /v1/join.
        """
        path = token_file or token_path_for(name)
        candidates: list[str] = []
        if not fresh:
            if token:
                candidates.append(token.strip())
            elif path is not None:
                saved = load_token(path)
                if saved:
                    candidates.append(saved)

        for candidate in candidates:
            try:
                obs = self.resume(candidate)
            except ValleyError:
                self.token = None
                continue
            save_token(path, candidate, name=obs["self_info"]["name"])
            return {
                "actor_id": None,
                "name": obs["self_info"]["name"],
                "token": candidate,
                "resumed": True,
                "token_file": str(path),
            }

        joined = self.join(name)
        save_token(path, joined["token"], name=joined["name"])
        return {
            **joined,
            "resumed": False,
            "token_file": str(path),
        }

    def clock(self) -> dict:
        return self._request("GET", "/v1/clock", None, auth=False)

    def observe(self) -> dict:
        return self._request("GET", "/v1/observe", None)

    def intent(self, line: str, command_id: str | None = None, window: int | None = None) -> dict:
        return self._request(
            "POST",
            "/v1/intent",
            {
                "command_id": command_id or str(uuid.uuid4()),
                "line": line,
                "window": window,
            },
        )

    def advance(self, steps: int = 1) -> dict:
        return self._request("POST", "/v1/advance", {"steps": steps}, auth=False)

    def _request(self, method: str, path: str, body: dict | None, auth: bool = True) -> dict:
        data = None if body is None else json.dumps(body).encode()
        request = urllib.request.Request(self.base + path, data=data, method=method)
        if data is not None:
            request.add_header("Content-Type", "application/json")
        if auth:
            if not self.token:
                raise ValleyError("join first")
            request.add_header("Authorization", f"Bearer {self.token}")
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                return json.loads(response.read().decode())
        except urllib.error.HTTPError as err:
            detail = err.read().decode()
            try:
                message = json.loads(detail).get("message", detail)
            except json.JSONDecodeError:
                message = detail
            raise ValleyError(message) from err
