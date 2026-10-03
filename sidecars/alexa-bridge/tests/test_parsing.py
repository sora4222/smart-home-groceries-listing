"""Unit tests for slot parsing — the awkward cases, without a skill request."""

import pytest

from alexa_bridge.parsing import (
    MAX_QUANTITY,
    UnusableRequest,
    parse_item_name,
    parse_optional_quantity,
    parse_quantity,
    parse_removal_slots,
    parse_slots,
)


def test_item_name_collapses_whitespace():
    assert parse_item_name("  full   cream  milk ") == "full cream milk"


def test_item_name_is_otherwise_left_as_heard():
    # Correcting a misheard item is the household member's job on the card.
    assert parse_item_name("mispelled itme") == "mispelled itme"


def test_item_name_is_truncated_to_the_backends_limit():
    assert len(parse_item_name("x" * 500)) == 200


@pytest.mark.parametrize("raw", [None, "", "   "])
def test_a_missing_item_name_is_unusable(raw):
    with pytest.raises(UnusableRequest):
        parse_item_name(raw)


@pytest.mark.parametrize("raw", [None, "", "   "])
def test_quantity_defaults_to_one(raw):
    assert parse_quantity(raw) == 1


def test_quantity_reads_digits():
    assert parse_quantity("3") == 3


def test_quantity_reads_number_words_alexa_did_not_resolve():
    assert parse_quantity("two") == 2
    assert parse_quantity("a") == 1
    assert parse_quantity("dozen") == 12


def test_quantity_falls_back_to_one_when_unparseable():
    # "the usual" is not a number, which is the same as not saying one.
    assert parse_quantity("the usual") == 1


def test_quantity_is_clamped_to_the_allowed_range():
    assert parse_quantity("100000") == MAX_QUANTITY
    assert parse_quantity("0") == 1
    assert parse_quantity("-5") == 1


def test_parse_slots_combines_both():
    parsed = parse_slots("oat milk", "2")
    assert parsed.item == "oat milk"
    assert parsed.quantity == 2


@pytest.mark.parametrize("raw", [None, "", "   ", "the usual"])
def test_an_unsaid_removal_quantity_stays_absent(raw):
    # "remove milk" means the whole item, not "one milk".
    assert parse_optional_quantity(raw) is None


def test_a_said_removal_quantity_is_read_and_clamped():
    assert parse_optional_quantity("2") == 2
    assert parse_optional_quantity("two") == 2
    assert parse_optional_quantity("100000") == MAX_QUANTITY
    assert parse_optional_quantity("0") == 1


def test_parse_removal_slots_keeps_an_absent_quantity_absent():
    parsed = parse_removal_slots(" eggs ", None)
    assert parsed.item == "eggs"
    assert parsed.quantity is None


def test_parse_removal_slots_needs_an_item():
    with pytest.raises(UnusableRequest):
        parse_removal_slots(None, "2")
