"""Integration tests for the Google Home voice-request confirmation queue."""

import pytest

pytestmark = pytest.mark.asyncio


async def test_webhook_requires_shared_secret(client, new_item_name):
    resp = await client.post(
        "/api/voice-requests", json={"item": new_item_name, "quantity": 1}
    )
    assert resp.status_code == 401


async def test_webhook_creates_pending_request(client, webhook_headers, new_item_name):
    resp = await client.post(
        "/api/voice-requests",
        json={"item": new_item_name, "quantity": 2},
        headers=webhook_headers,
    )
    assert resp.status_code == 201
    body = resp.json()
    assert body["parsed_name"] == new_item_name
    assert body["parsed_quantity"] == 2
    assert body["status"] == "pending"


async def test_webhook_defaults_quantity_to_one(client, webhook_headers, new_item_name):
    resp = await client.post(
        "/api/voice-requests", json={"item": new_item_name}, headers=webhook_headers
    )
    assert resp.status_code == 201
    assert resp.json()["parsed_quantity"] == 1


async def test_pending_list_requires_auth_but_shows_unconfirmed_items(
    client, webhook_headers, new_item_name
):
    await client.post(
        "/api/voice-requests",
        json={"item": new_item_name, "quantity": 1},
        headers=webhook_headers,
    )
    resp = await client.get("/api/voice-requests")
    assert resp.status_code == 200
    names = [r["parsed_name"] for r in resp.json()]
    assert new_item_name in names


async def test_accept_moves_item_to_active_grocery_list(
    client, webhook_headers, new_item_name
):
    create = await client.post(
        "/api/voice-requests",
        json={"item": new_item_name, "quantity": 3},
        headers=webhook_headers,
    )
    request_id = create.json()["id"]

    resp = await client.post(f"/api/voice-requests/{request_id}/accept")
    assert resp.status_code == 200
    body = resp.json()
    assert body["voice_request"]["status"] == "accepted"
    assert body["grocery_item"]["name"] == new_item_name
    assert body["grocery_item"]["quantity"] == 3
    assert body["grocery_item"]["status"] == "active"
    assert body["grocery_item"]["source"] == "voice"

    listed = await client.get("/api/grocery-items")
    assert any(i["name"] == new_item_name for i in listed.json())


async def test_accept_applies_user_corrections(client, webhook_headers, new_item_name):
    create = await client.post(
        "/api/voice-requests",
        json={"item": "mispelled itme", "quantity": 1},
        headers=webhook_headers,
    )
    request_id = create.json()["id"]

    resp = await client.post(
        f"/api/voice-requests/{request_id}/accept",
        json={"name": new_item_name, "quantity": 5},
    )
    assert resp.status_code == 200
    assert resp.json()["grocery_item"]["name"] == new_item_name
    assert resp.json()["grocery_item"]["quantity"] == 5


async def test_reject_discards_the_request(client, webhook_headers, new_item_name):
    create = await client.post(
        "/api/voice-requests",
        json={"item": new_item_name, "quantity": 1},
        headers=webhook_headers,
    )
    request_id = create.json()["id"]

    resp = await client.post(f"/api/voice-requests/{request_id}/reject")
    assert resp.status_code == 200
    assert resp.json()["status"] == "rejected"

    pending = await client.get("/api/voice-requests")
    assert request_id not in [r["id"] for r in pending.json()]


async def test_accepting_a_rejected_request_undoes_the_rejection(
    client, webhook_headers, new_item_name
):
    """Supports the "dulled, undo-on-hover" reject UX in the web app."""
    create = await client.post(
        "/api/voice-requests",
        json={"item": new_item_name, "quantity": 1},
        headers=webhook_headers,
    )
    request_id = create.json()["id"]
    await client.post(f"/api/voice-requests/{request_id}/reject")

    resp = await client.post(f"/api/voice-requests/{request_id}/accept")
    assert resp.status_code == 200
    assert resp.json()["voice_request"]["status"] == "accepted"


async def test_cannot_accept_an_already_accepted_request(
    client, webhook_headers, new_item_name
):
    create = await client.post(
        "/api/voice-requests",
        json={"item": new_item_name, "quantity": 1},
        headers=webhook_headers,
    )
    request_id = create.json()["id"]
    await client.post(f"/api/voice-requests/{request_id}/accept")

    resp = await client.post(f"/api/voice-requests/{request_id}/accept")
    assert resp.status_code == 409


async def test_accepting_a_duplicate_active_item_returns_409_with_existing_item(
    client, webhook_headers, new_item_name
):
    first = await client.post(
        "/api/voice-requests",
        json={"item": new_item_name, "quantity": 1},
        headers=webhook_headers,
    )
    await client.post(f"/api/voice-requests/{first.json()['id']}/accept")

    second = await client.post(
        "/api/voice-requests",
        json={"item": new_item_name.upper(), "quantity": 2},
        headers=webhook_headers,
    )
    resp = await client.post(f"/api/voice-requests/{second.json()['id']}/accept")
    assert resp.status_code == 409
    assert "already on the list" in resp.json()["detail"]["message"]


async def test_merge_flag_increments_existing_item_quantity_instead_of_duplicating(
    client, webhook_headers, new_item_name
):
    first = await client.post(
        "/api/voice-requests",
        json={"item": new_item_name, "quantity": 1},
        headers=webhook_headers,
    )
    await client.post(f"/api/voice-requests/{first.json()['id']}/accept")

    second = await client.post(
        "/api/voice-requests",
        json={"item": new_item_name, "quantity": 2},
        headers=webhook_headers,
    )
    resp = await client.post(
        f"/api/voice-requests/{second.json()['id']}/accept?merge=true"
    )
    assert resp.status_code == 200
    assert resp.json()["grocery_item"]["quantity"] == 3

    listed = (await client.get("/api/grocery-items")).json()
    matching = [i for i in listed if i["name"] == new_item_name]
    assert len(matching) == 1
