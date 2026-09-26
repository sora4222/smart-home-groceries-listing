"""Unit tests for the pure name-normalisation helper used in duplicate detection."""

from app.services.voice import _normalise


def test_normalise_lowercases_and_trims():
    assert _normalise("  Toilet Paper  ") == "toilet paper"


def test_normalise_collapses_internal_whitespace():
    assert _normalise("full   cream   milk") == "full cream milk"


def test_normalise_is_case_and_space_insensitive_for_matching():
    assert _normalise("Milk") == _normalise("  milk ")
