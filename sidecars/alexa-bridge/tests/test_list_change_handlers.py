"""Tests for the remove, reduce and undo intent handlers.

The handlers run against a fake client, so these check how slots are read
and what Alexa says, with no backend and no Amazon.
"""

import pytest
from ask_sdk_core.handler_input import HandlerInput
from ask_sdk_model import Intent, IntentRequest, RequestEnvelope, Slot

from alexa_bridge.backend import BackendUnavailable
from alexa_bridge.list_change_handlers import RemoveItemIntentHandler, UndoIntentHandler
from alexa_bridge.list_changes import ChangeRefused, ListChange, NotOnList

REMOVED = ListChange(kind="removed", item_name="milk", quantity_before=2, quantity_after=0)


class FakeClient:
    """Records what the handler forwarded; answers or raises as told."""

    def __init__(self, answer=REMOVED, raises=None):
        self.answer = answer
        self.raises = raises
        self.removed = []
        self.undone = []

    def remove_item(self, parsed, request_id):
        self.removed.append((parsed, request_id))
        if self.raises:
            raise self.raises
        return self.answer

    def undo(self, request_id):
        self.undone.append(request_id)
        if self.raises:
            raise self.raises
        return self.answer


def intent(name, **slots):
    """A HandlerInput for an IntentRequest with these filled slots."""
    filled = {key: Slot(name=key, value=value) for key, value in slots.items()}
    request = IntentRequest(request_id="req-1", intent=Intent(name=name, slots=filled))
    return HandlerInput(request_envelope=RequestEnvelope(request=request))


def spoken(response):
    return response.output_speech.ssml.removeprefix("<speak>").removesuffix("</speak>")


def test_remove_with_no_number_forwards_the_whole_item():
    client = FakeClient()
    RemoveItemIntentHandler(client).handle(intent("RemoveItemIntent", item="milk"))
    parsed, request_id = client.removed[0]
    assert (parsed.item, parsed.quantity, request_id) == ("milk", None, "req-1")


def test_remove_with_a_number_forwards_it():
    client = FakeClient()
    RemoveItemIntentHandler(client).handle(intent("RemoveItemIntent", item="milk", quantity="2"))
    assert client.removed[0][0].quantity == 2


def test_reduce_with_no_number_takes_one_off():
    client = FakeClient()
    RemoveItemIntentHandler(client).handle(intent("ReduceItemIntent", item="milk"))
    assert client.removed[0][0].quantity == 1


def test_a_change_is_confirmed_and_the_session_stays_open_for_undo():
    response = RemoveItemIntentHandler(FakeClient()).handle(intent("RemoveItemIntent", item="milk"))
    assert spoken(response) == "Removed milk from the list. Say undo to put it back."
    assert response.should_end_session is False


def test_a_missing_item_asks_again_and_forwards_nothing():
    client = FakeClient()
    response = RemoveItemIntentHandler(client).handle(intent("RemoveItemIntent"))
    assert client.removed == []
    assert "didn't catch the item" in spoken(response)


@pytest.mark.parametrize(
    ("error", "says"),
    [
        (NotOnList(), "I couldn't find milk on the list, so nothing changed."),
        (ChangeRefused(), "The list is locked for purchase"),
        (BackendUnavailable("down"), "couldn't reach the shopping list"),
    ],
)
def test_a_failed_remove_says_why_and_never_claims_success(error, says):
    response = RemoveItemIntentHandler(FakeClient(raises=error)).handle(
        intent("RemoveItemIntent", item="milk")
    )
    assert says in spoken(response)
    assert "Removed" not in spoken(response)


def test_undo_forwards_the_request_id_and_confirms():
    client = FakeClient()
    response = UndoIntentHandler(client).handle(intent("UndoIntent"))
    assert client.undone == ["req-1"]
    assert spoken(response) == "Put milk back on the list."


@pytest.mark.parametrize(
    ("error", "says"),
    [
        (NotOnList(), "There's nothing recent to undo."),
        (ChangeRefused(), "I couldn't undo that"),
        (BackendUnavailable("down"), "couldn't reach the shopping list"),
    ],
)
def test_a_failed_undo_says_why(error, says):
    response = UndoIntentHandler(FakeClient(raises=error)).handle(intent("UndoIntent"))
    assert says in spoken(response)


def test_the_handlers_claim_only_their_intents():
    remove = RemoveItemIntentHandler(FakeClient())
    undo = UndoIntentHandler(FakeClient())
    assert remove.can_handle(intent("RemoveItemIntent"))
    assert remove.can_handle(intent("ReduceItemIntent"))
    assert not remove.can_handle(intent("AddItemIntent"))
    assert undo.can_handle(intent("UndoIntent"))
    assert not undo.can_handle(intent("RemoveItemIntent"))
