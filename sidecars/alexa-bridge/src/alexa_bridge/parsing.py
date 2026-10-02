"""Turning Alexa intent slots into an item name and a quantity.

Pure functions, so the awkward cases (a missing slot, a quantity Alexa could
not resolve, a number outside what the list allows) are unit-testable without
a skill request.
"""

from __future__ import annotations

from dataclasses import dataclass

#: Bounds the backend also enforces. Clamping here keeps a misheard "a
#: thousand" from being rejected outright — the household member sees the
#: capped number on the confirmation card and can correct it.
MIN_QUANTITY = 1
MAX_QUANTITY = 999

#: Small number words, for the cases where Alexa passes the spoken form
#: through instead of resolving it to digits.
_NUMBER_WORDS = {
    "a": 1,
    "an": 1,
    "one": 1,
    "two": 2,
    "three": 3,
    "four": 4,
    "five": 5,
    "six": 6,
    "seven": 7,
    "eight": 8,
    "nine": 9,
    "ten": 10,
    "eleven": 11,
    "twelve": 12,
    "dozen": 12,
}


class UnusableRequest(ValueError):
    """The utterance carried no item name, so there is nothing to record."""


@dataclass(frozen=True)
class ParsedItem:
    """What the bridge forwards to the backend."""

    item: str
    quantity: int


@dataclass(frozen=True)
class ParsedRemoval:
    """What the bridge forwards to take an item off, or lower its quantity.

    `quantity` of `None` means the whole item; a number means "this many
    fewer". What either does to the list is the backend's decision.
    """

    item: str
    quantity: int | None


def parse_item_name(raw: str | None) -> str:
    """Normalises a spoken item name.

    Whitespace is collapsed and the result truncated to the backend's limit.
    The name is otherwise left exactly as Alexa heard it — correcting a
    misheard item is the household member's job on the confirmation card, not
    this bridge's.

    Raises:
        UnusableRequest: if no usable name was supplied.
    """
    name = " ".join((raw or "").split())
    if not name:
        raise UnusableRequest("no item name in the request")
    return name[:200]


def parse_quantity(raw: str | None) -> int:
    """Reads a quantity slot, defaulting to one and clamping to the allowed range.

    Alexa normally resolves `AMAZON.NUMBER` to digits, but an unresolved slot
    arrives as the spoken word, and a slot the user never filled arrives as
    `None` or an empty string.
    """
    text = (raw or "").strip().lower()
    if not text:
        return MIN_QUANTITY

    if text in _NUMBER_WORDS:
        value = _NUMBER_WORDS[text]
    else:
        try:
            value = int(float(text))
        except ValueError:
            # Anything unparseable means "the user did not usefully say a
            # number", which is the same situation as not saying one.
            return MIN_QUANTITY

    return max(MIN_QUANTITY, min(value, MAX_QUANTITY))


def parse_optional_quantity(raw: str | None) -> int | None:
    """Reads a quantity slot that may rightly be empty.

    `None` when no usable number was said ("remove milk"), otherwise the
    number clamped like `parse_quantity`. Unlike adding, an unheard number
    is not read as one: "remove milk" and "remove one milk" mean different
    things.
    """
    text = (raw or "").strip().lower()
    if not text:
        return None
    if text not in _NUMBER_WORDS:
        try:
            int(float(text))
        except ValueError:
            return None
    return parse_quantity(text)


def parse_slots(item: str | None, quantity: str | None) -> ParsedItem:
    """Parses both slots together.

    Raises:
        UnusableRequest: if there is no item name.
    """
    return ParsedItem(item=parse_item_name(item), quantity=parse_quantity(quantity))


def parse_removal_slots(item: str | None, quantity: str | None) -> ParsedRemoval:
    """Parses a remove request: the item, and a quantity only if one was said.

    Raises:
        UnusableRequest: if there is no item name.
    """
    return ParsedRemoval(item=parse_item_name(item), quantity=parse_optional_quantity(quantity))
