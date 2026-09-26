"""WebSocket endpoint pushing real-time events to open browser sessions.

Currently only `voice_request_added` is emitted (see `services/ws_manager.py`
and `services/voice.py`). The frontend uses this to update the Pending
Requests badge and fire a Sonner toast without polling.
"""

from fastapi import APIRouter, WebSocket, WebSocketDisconnect

from app.services.ws_manager import manager

router = APIRouter(tags=["ws"])


@router.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket) -> None:
    await manager.connect(websocket)
    try:
        while True:
            # The client does not send anything meaningful; this just
            # keeps the connection open and detects disconnects.
            await websocket.receive_text()
    except WebSocketDisconnect:
        manager.disconnect(websocket)
