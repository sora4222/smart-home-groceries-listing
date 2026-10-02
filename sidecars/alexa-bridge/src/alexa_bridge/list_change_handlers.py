"""Intent handlers for taking items off the list, and undoing that.

Like `handlers.py`, these only read slots, forward them and speak the
backend's answer. Which item matches, what a quantity does and what Undo
reverses are all decided in the Rust backend.

After a change the session stays open, so the household member can just say
"undo" without starting the skill again.
"""

from __future__ import annotations

import logging

from ask_sdk_core.dispatch_components import AbstractRequestHandler
from ask_sdk_core.handler_input import HandlerInput
from ask_sdk_core.utils import get_slot_value, is_intent_name
from ask_sdk_model import Response

from alexa_bridge import speech
from alexa_bridge.backend import BackendUnavailable
from alexa_bridge.list_changes import ChangeRefused, ListChangeClient, NotOnList
from alexa_bridge.parsing import ParsedRemoval, UnusableRequest, parse_removal_slots, parse_slots

logger = logging.getLogger(__name__)

#: "remove milk", "remove two milk" — no quantity means the whole item.
REMOVE_ITEM_INTENT = "RemoveItemIntent"
#: "reduce milk by two", "reduce milk" — no quantity means one fewer.
REDUCE_ITEM_INTENT = "ReduceItemIntent"
#: "undo", "put it back".
UNDO_INTENT = "UndoIntent"
#: Slot names, shared with `AddItemIntent`.
ITEM_SLOT = "item"
QUANTITY_SLOT = "quantity"

#: Said when the backend could not be reached.
UNREACHABLE = "Sorry, I couldn't reach the shopping list just now. Please try again in a moment."
#: Said when the item slot was empty.
NO_ITEM = "I didn't catch the item. Try saying, remove milk from the shopping list."


class RemoveItemIntentHandler(AbstractRequestHandler):
    """Handles "remove milk" and "reduce milk by two"."""

    def __init__(self, client: ListChangeClient) -> None:
        self._client = client

    def can_handle(self, handler_input: HandlerInput) -> bool:
        return is_intent_name(REMOVE_ITEM_INTENT)(handler_input) or is_intent_name(
            REDUCE_ITEM_INTENT
        )(handler_input)

    def handle(self, handler_input: HandlerInput) -> Response:
        builder = handler_input.response_builder
        try:
            parsed = _parse(handler_input)
        except UnusableRequest:
            return builder.speak(NO_ITEM).set_should_end_session(False).response

        request_id = handler_input.request_envelope.request.request_id
        try:
            change = self._client.remove_item(parsed, request_id=request_id)
        except NotOnList:
            spoken = speech.not_on_list(parsed.item)
            return builder.speak(spoken).set_should_end_session(True).response
        except ChangeRefused:
            return builder.speak(speech.LIST_LOCKED).set_should_end_session(True).response
        except BackendUnavailable as exc:
            logger.warning("list change not made: %s", exc)
            return builder.speak(UNREACHABLE).set_should_end_session(True).response

        # Left open so "undo" works straight away; Alexa closes it on silence.
        return builder.speak(speech.change_made(change)).set_should_end_session(False).response


class UndoIntentHandler(AbstractRequestHandler):
    """Handles "undo": reverses the newest remove or reduce."""

    def __init__(self, client: ListChangeClient) -> None:
        self._client = client

    def can_handle(self, handler_input: HandlerInput) -> bool:
        return is_intent_name(UNDO_INTENT)(handler_input)

    def handle(self, handler_input: HandlerInput) -> Response:
        builder = handler_input.response_builder
        request_id = handler_input.request_envelope.request.request_id
        try:
            change = self._client.undo(request_id=request_id)
        except NotOnList:
            spoken = speech.NOTHING_TO_UNDO
        except ChangeRefused:
            spoken = speech.CANNOT_UNDO
        except BackendUnavailable as exc:
            logger.warning("undo not made: %s", exc)
            spoken = UNREACHABLE
        else:
            spoken = speech.change_undone(change)
        return builder.speak(spoken).set_should_end_session(True).response


def _parse(handler_input: HandlerInput) -> ParsedRemoval:
    """Reads the slots the way the intent means them.

    Remove with no number takes the whole item; reduce with no number takes
    one off.
    """
    item = get_slot_value(handler_input, ITEM_SLOT)
    quantity = get_slot_value(handler_input, QUANTITY_SLOT)
    if is_intent_name(REDUCE_ITEM_INTENT)(handler_input):
        parsed = parse_slots(item, quantity)
        return ParsedRemoval(item=parsed.item, quantity=parsed.quantity)
    return parse_removal_slots(item, quantity)
