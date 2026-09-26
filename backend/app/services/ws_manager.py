"""In-memory WebSocket connection registry and broadcast helper.

A single process-wide `manager` instance tracks open browser sessions and
pushes JSON events to all of them (e.g. when a voice request arrives).
Not shared across multiple backend processes — fine for the single-machine
home-server deployment this app targets.
"""

import json
from typing import Any

from fastapi import WebSocket


class ConnectionManager:
    def __init__(self) -> None:
        self._connections: set[WebSocket] = set()

    async def connect(self, websocket: WebSocket) -> None:
        await websocket.accept()
        self._connections.add(websocket)

    def disconnect(self, websocket: WebSocket) -> None:
        self._connections.discard(websocket)

    async def broadcast(self, event: dict[str, Any]) -> None:
        """Send a JSON event to every connected session, dropping dead ones."""
        payload = json.dumps(event)
        dead: list[WebSocket] = []
        for connection in self._connections:
            try:
                await connection.send_text(payload)
            except Exception:  # noqa: BLE001 — a dead socket must not stop the broadcast
                dead.append(connection)
        for connection in dead:
            self.disconnect(connection)


manager = ConnectionManager()
