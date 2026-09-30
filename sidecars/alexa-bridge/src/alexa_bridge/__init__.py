"""Alexa bridge: verify Alexa's requests, forward the item, hold no state.

This sidecar exists for one reason: Amazon's request signing and skill
request model are only implemented in the `ask-sdk` Python SDK, and there is
no maintained Rust equivalent. Everything else in the backend is Rust.

It deliberately contains no business logic. Duplicate handling, the
confirmation queue and the grocery list all live in the Rust backend; this
process verifies the caller is really Alexa, pulls the item and quantity out
of the intent, and posts them on.
"""

__all__ = ["__version__"]

__version__ = "0.1.0"
