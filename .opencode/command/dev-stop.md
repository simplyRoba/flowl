---
description: Stop the running hot-reload dev servers (Vite UI + Rust backend).
---

Stop the dev servers that were started by `/dev-start`.

List all running background tasks and stop any that match:
- The Vite UI dev server (`npm run dev --prefix ui`)
- The Rust backend (`cargo watch -x run`)

Confirm to the user which tasks were stopped.
