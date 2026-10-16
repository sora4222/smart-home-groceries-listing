/** Normalize framework-owned reports; diagnostic stdout never supplies verdicts. */
import { readFileSync } from 'node:fs';

/** Reject incomplete reports, failures, skips, and suites that execute no tests. */
export function verifyResults(label, result) {
  for (const key of ['passed', 'failed', 'skipped']) {
    if (!Number.isSafeInteger(result[key]) || result[key] < 0) throw new Error(`${label}: invalid result report`);
  }
  if (result.failed || result.skipped || !result.passed) {
    throw new Error(`${label}: ${result.passed} passed, ${result.failed} failed, ${result.skipped} skipped`);
  }
  console.log(`${label}: ${result.passed} passed`);
}

/** Translate dedicated framework reports into the shared count contract. */
export function readResults(framework, file) {
  const contents = readFileSync(file, 'utf8');
  if (framework === 'libtest') {
    const result = { passed: 0, failed: 0, skipped: 0 };
    for (const line of contents.split('\n').filter(Boolean)) {
      const status = line.split(' ')[0];
      const key = { ok: 'passed', failed: 'failed', ignored: 'skipped' }[status];
      if (!key) throw new Error('Invalid libtest record');
      result[key]++;
    }
    return result;
  }
  const report = JSON.parse(contents);
  switch (framework) {
    case 'vitest': return { passed: report.numPassedTests, failed: report.numFailedTests, skipped: report.numPendingTests + report.numTodoTests };
    case 'pytest': return report;
    case 'playwright': return {
      passed: report.stats.expected,
      failed: report.stats.unexpected + report.stats.flaky + (report.errors?.length ?? 0),
      skipped: report.stats.skipped,
    };
    default: throw new Error(`Unknown test framework: ${framework}`);
  }
}
