---
name: human-instructions
description: Write developer-facing documentation or plain chat steps for actions only a person can complete, such as account setup, console changes, secrets, logins, browser prompts, or Alexa interaction. Excludes developer checks and ordinary commands.
---

# Human steps

Use this skill for actions the agent cannot perform. Put one-time provisioning
in `docs/human-setup.md`, recurring use in `docs/using-the-app.md`, and
feature rationale in the relevant `docs/features/FEATURE_*.md` spec. Read the
target document before editing and place the instruction in its existing
section.

For repository documentation:

- Write for software developers and use precise UI labels and technical terms.
- Group steps by place (shell, vendor console, web app, device), then order
  them by dependency. Collect configuration values before the first startup.
- Use numbered steps, show value transfers in a table, name environment keys
  but never their secret values, and add a brief observable verification.
- Put security warnings immediately before the affected action. Keep optional
  work last and link to setup or operations docs rather than duplicating steps.

For chat, lead with the outcome and end with a short numbered list of what the
person must do. Ask only for logins, payments, approvals, physical devices, or
other human-only actions.
