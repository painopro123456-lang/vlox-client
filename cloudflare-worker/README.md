# VLOX CurseForge API Worker

This Worker keeps the CurseForge API key off client machines. VLOX calls this service; the Worker adds the key from its encrypted Cloudflare secret.

## Deploy once

1. Create or sign in to a Cloudflare account.
2. Install Node.js, then run `npm install -g wrangler` and `wrangler login`.
3. From this folder, set the secret: `npx wrangler secret put CURSEFORGE_API_KEY`. Paste your CurseForge key into the hidden prompt. Do not commit it to GitHub.
4. Deploy: `npx wrangler deploy`.
5. Cloudflare will show a URL like `https://vlox-curseforge-proxy.<your-account-subdomain>.workers.dev`.
6. Update `CURSEFORGE_PROXY_BASE` in `src/library.rs` to that exact URL, commit, and build VLOX again.

## Endpoints

- `GET /v1/mods/search?searchFilter=...`
- `GET /v1/mods/{projectId}/files`
- `GET /v1/mods/{projectId}/files/{fileId}`

The Worker only exposes these read-only CurseForge API routes and fixes searches to Minecraft 26.2 + Fabric. It does not expose the secret to the desktop app.

## Important

A public proxy can be abused by third parties and can consume your CurseForge API quota. Before distributing widely, add Cloudflare rate limiting and an abuse-control policy. Do not add the API key to VLOX source, app settings, or GitHub Actions logs.
