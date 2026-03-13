---
description: Stop the running hot-reload dev servers (Vite UI + Rust backend).
---

Stop the dev servers that were started by `/dev-start`.

## Steps

1. **Check Claude background tasks**: Use `TaskList` to list all tracked tasks. Stop any that match the Vite UI dev server or Rust backend using `TaskStop`.

2. **Check OS processes**: Run `ps aux | grep -E "cargo watch|vite|npm run dev" | grep -v grep` to find any remaining processes. Kill them with `kill <pid>`.

3. **Verify**: Run the same `ps` command again to confirm everything is stopped.

4. **Report**: Tell the user what was stopped. If nothing was found in either step, say so.
