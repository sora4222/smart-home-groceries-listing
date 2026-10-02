"""Intent handlers for the grocery skill.

Each handler does as little as possible: pull the slots out, forward them,
say what happened. Anything that looks like a rule about the grocery list
belongs in the Rust backend instead.

Nothing here suggests, recommends or upsells an item. Reducing
marketing-driven buying is a goal of the project, so the skill only ever
confirms what the household member actually asked for.
"""

from __future__ import annotations

import logging

from ask_sdk_core.dispatch_components import AbstractExceptionHandler, AbstractRequestHandler
from ask_sdk_core.handler_input import HandlerInput
from ask_sdk_core.utils import get_slot_value, is_intent_name, is_request_type
from ask_sdk_model import Response

from alexa_bridge.backend import BackendClient, BackendUnavailable
from alexa_bridge.parsing import UnusableRequest, parse_slots

logger = logging.getLogger(__name__)

#: The intent that adds an item, as named in the skill's interaction model.
ADD_ITEM_INTENT = "AddItemIntent"
#: Slot names in that intent.
ITEM_SLOT = "item"
QUANTITY_SLOT = "quantity"


class AddItemIntentHandler(AbstractRequestHandler):
    """Handles "Alexa, add two litres of milk to the shopping list"."""

    def __init__(self, client: BackendClient) -> None:
        self._client = client

    def can_handle(self, handler_input: HandlerInput) -> bool:
        return is_intent_name(ADD_ITEM_INTENT)(handler_input)

    def handle(self, handler_input: HandlerInput) -> Response:
        try:
            parsed = parse_slots(
                get_slot_value(handler_input, ITEM_SLOT),
                get_slot_value(handler_input, QUANTITY_SLOT),
            )
        except UnusableRequest:
            return (
                handler_input.response_builder.speak(
                    "I didn't catch the item. Try saying, add milk to the shopping list."
                )
                .set_should_end_session(False)
                .response
            )

        request_id = handler_input.request_envelope.request.request_id
        raw_text = _raw_utterance(handler_input)

        try:
            self._client.record_item(parsed, request_id=request_id, raw_text=raw_text)
        except BackendUnavailable as exc:
            logger.warning("item not recorded: %s", exc)
            return (
                handler_input.response_builder.speak(
                    "Sorry, I couldn't reach the shopping list just now. "
                    "Please try again in a moment."
                )
                .set_should_end_session(True)
                .response
            )

        # Deliberately says "for review", not "added": the item is pending
        # until a household member accepts it in the web app.
        spoken = _confirmation(parsed.item, parsed.quantity)
        return handler_input.response_builder.speak(spoken).set_should_end_session(True).response


class LaunchRequestHandler(AbstractRequestHandler):
    """Handles "Alexa, open the shopping list" with no item."""

    def can_handle(self, handler_input: HandlerInput) -> bool:
        return is_request_type("LaunchRequest")(handler_input)

    def handle(self, handler_input: HandlerInput) -> Response:
        return (
            handler_input.response_builder.speak(
                "What would you like to add to or remove from the shopping list?"
            )
            .ask("What should I add or remove?")
            .response
        )


class HelpIntentHandler(AbstractRequestHandler):
    """Handles the built-in help intent."""

    def can_handle(self, handler_input: HandlerInput) -> bool:
        return is_intent_name("AMAZON.HelpIntent")(handler_input)

    def handle(self, handler_input: HandlerInput) -> Response:
        return (
            handler_input.response_builder.speak(
                "Say something like, add milk to the shopping list. "
                "Items wait for someone to confirm them in the web app. "
                "You can also say, remove milk, or reduce milk by two, "
                "and say undo to put it back."
            )
            .ask("What should I add or remove?")
            .response
        )


class CancelAndStopIntentHandler(AbstractRequestHandler):
    """Handles the built-in cancel and stop intents."""

    def can_handle(self, handler_input: HandlerInput) -> bool:
        return is_intent_name("AMAZON.CancelIntent")(handler_input) or is_intent_name(
            "AMAZON.StopIntent"
        )(handler_input)

    def handle(self, handler_input: HandlerInput) -> Response:
        return handler_input.response_builder.speak("Okay.").set_should_end_session(True).response


class SessionEndedRequestHandler(AbstractRequestHandler):
    """Acknowledges session end; there is no state to clean up."""

    def can_handle(self, handler_input: HandlerInput) -> bool:
        return is_request_type("SessionEndedRequest")(handler_input)

    def handle(self, handler_input: HandlerInput) -> Response:
        return handler_input.response_builder.response


class CatchAllExceptionHandler(AbstractExceptionHandler):
    """Last resort, so an unexpected fault is never a silent failure.

    The exception is logged for the operator; the user hears a plain apology
    with no internal detail in it.
    """

    def can_handle(self, handler_input: HandlerInput, exception: Exception) -> bool:
        return True

    def handle(self, handler_input: HandlerInput, exception: Exception) -> Response:
        logger.exception("unhandled error in the Alexa bridge", exc_info=exception)
        return (
            handler_input.response_builder.speak("Sorry, something went wrong. Please try again.")
            .set_should_end_session(True)
            .response
        )


def _confirmation(item: str, quantity: int) -> str:
    """The sentence Alexa speaks back."""
    if quantity == 1:
        return f"Added {item} to the list for review."
    return f"Added {quantity} {item} to the list for review."


def _raw_utterance(handler_input: HandlerInput) -> str | None:
    """Best available text of what was said, for the confirmation card.

    Alexa does not hand a custom skill the full transcript, so this
    reconstructs it from the filled slots. It is shown to the household
    member as context, never parsed by the backend.
    """
    try:
        intent = handler_input.request_envelope.request.intent
        slots = intent.slots or {}
        values = [slot.value for slot in slots.values() if slot and slot.value]
    except AttributeError:
        return None
    return " ".join(values) or None
