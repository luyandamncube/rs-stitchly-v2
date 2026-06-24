# Stitchly TikTok OAuth Postbox

This is the Vercel side of the no-copy/paste TikTok OAuth test flow.

It stores a TikTok authorization `code` temporarily by `state`, so local scripts
can poll for the code and exchange it locally. Client secrets and access tokens
stay on your machine.

## Deploy

From this folder:

```bash
npm install
vercel --prod
```

In Vercel, attach a KV store to the project so `@vercel/kv` has the required
environment variables.

Register this redirect URI in TikTok Login Kit:

```text
https://YOUR-VERCEL-PROJECT.vercel.app/api/callback
```

Set local `docs/tiktok/.env.tiktok`:

```bash
export TIKTOK_REDIRECT_URI="https://YOUR-VERCEL-PROJECT.vercel.app/api/callback"
export TIKTOK_CALLBACK_CODE_URL="https://YOUR-VERCEL-PROJECT.vercel.app/api/code"
```

## Local Flow

```bash
bash docs/tiktok/tiktok_login_exchange.sh
bash docs/tiktok/tiktok_auth_probe.sh
```
