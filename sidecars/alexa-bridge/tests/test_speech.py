"""Tests for what Alexa says after a remove, reduce or undo."""

from alexa_bridge import speech
from alexa_bridge.list_changes import ListChange

REMOVED = ListChange(kind="removed", item_name="milk", quantity_before=2, quantity_after=0)
REDUCED = ListChange(kind="reduced", item_name="eggs", quantity_before=6, quantity_after=4)


def test_a_removal_is_confirmed_with_how_to_undo_it():
    assert speech.change_made(REMOVED) == ("Removed milk from the list. Say undo to put it back.")


def test_a_reduction_says_the_new_quantity():
    assert speech.change_made(REDUCED) == (
        "eggs is down to 4 on the list. Say undo to change it back."
    )


def test_undoing_a_removal_says_it_is_back():
    assert speech.change_undone(REMOVED) == "Put milk back on the list."


def test_undoing_a_reduction_says_how_many_came_back():
    assert speech.change_undone(REDUCED) == "Added 2 eggs back to the list."


def test_an_unmatched_item_says_nothing_changed():
    assert speech.not_on_list("bread") == "I couldn't find bread on the list, so nothing changed."
