"""Write pytest outcomes separately from captured application output."""

import json
import os
from pathlib import Path

import pytest


@pytest.hookimpl(trylast=True)
def pytest_sessionfinish(session, exitstatus):
    """Include setup errors, expected failures, skips, and unexpected passes."""
    reporter = session.config.pluginmanager.get_plugin("terminalreporter")
    stats = reporter.stats
    Path(os.environ["GROCERY_TEST_REPORT"]).write_text(
        json.dumps(
            {
                "passed": len(stats.get("passed", [])),
                "failed": len(stats.get("failed", []))
                + len(stats.get("error", []))
                + len(stats.get("xpassed", []))
                + int(exitstatus not in (0, 1, 5)),
                "skipped": len(stats.get("skipped", [])) + len(stats.get("xfailed", [])),
            }
        )
    )
