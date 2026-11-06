"""Player API client. Humans and this process use the same routes."""

from __future__ import annotations

import json
import urllib.error
import urllib.request
import uuid


class ValleyError(RuntimeError):
    pass


class Client:
    def __init__(self, base: str, token: str | None = None) -> None:
        self.base = base.rstrip("/")
        self.token = token

    def join(self, name: str) -> dict:
        body = self._request("POST", "/v1/join", {"name": name}, auth=False)
        self.token = body["token"]
        return body

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
