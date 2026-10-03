"""What Alexa says back after a remove, a reduce or an undo.

Pure functions over the backend's answer, so every sentence is unit-tested.
The backend decides what happened; these only put it into words. None of
them suggests an item.
"""

from __future__ import annotations

from alexa_bridge.list_changes import ListChange

#: Said when no item on the list matched the spoken name.
NOT_ON_LIST = "I couldn't find {item} on the list, so nothing changed."
#: Said when the list is committed for purchase.
LIST_LOCKED = (
    "The list is locked for purchase, so nothing changed. Release it in the web app first."
)
#: Said when there is nothing recent to undo.
NOTHING_TO_UNDO = "There's nothing recent to undo."
#: Said when the newest change can no longer be reversed.
CANNOT_UNDO = "I couldn't undo that, because the item was deleted or locked since."


def change_made(change: ListChange) -> str:
    """Confirms a remove or reduce, and says how to reverse it."""
    if change.kind == "removed":
        return f"Removed {change.item_name} from the list. Say undo to put it back."
    return (
        f"{change.item_name} is down to {change.quantity_after} on the list. "
        "Say undo to change it back."
    )


def change_undone(change: ListChange) -> str:
    """Confirms an undo."""
    if change.kind == "removed":
        return f"Put {change.item_name} back on the list."
    added_back = change.quantity_before - change.quantity_after
    return f"Added {added_back} {change.item_name} back to the list."


def not_on_list(item: str) -> str:
    """Says the spoken item matched nothing on the list."""
    return NOT_ON_LIST.format(item=item)
