---
name: minimal-changes-only
description: User wants minimal code changes - preserve comments, TODOs, logic, and structure. Only change what is strictly necessary for the task.
type: feedback
---

When porting or adapting code, only change what is strictly necessary (e.g., HAL-specific types, imports, API calls). Do NOT remove TODOs, comments, or restructure logic. Preserve the original code structure as closely as possible.

**Why:** The user values their existing comments, TODOs, and code structure. Removing them or "cleaning up" is unwanted.

**How to apply:** When adapting code to a new platform, diff against the original and ensure only hardware-specific lines change. Keep all comments, TODOs, variable names, and logic flow identical.
