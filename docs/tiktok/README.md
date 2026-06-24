# TikTok Auth Test Scripts

These scripts are a lightweight auth and API probe layer for TikTok before we
turn anything into a Stitchly workflow or connector.

They are designed for the Vercel callback flow that worked in the separate
Threadback repo:

```text
TikTok Login Kit
-> Vercel HTTPS callback
-> copy code
-> local token exchange
-> local user/video API probes
```

The primary scripts are shell scripts using `bash`, `curl`, and `jq`. Python
equivalents are also kept in the same folder as reference/fallback utilities.

## Setup

```bash
cp docs/tiktok/.env.example docs/tiktok/.env.tiktok
```

Edit `docs/tiktok/.env.tiktok`.

The shell scripts load this file automatically. You can still override the env
file path with `TIKTOK_ENV_FILE=/path/to/env`.

For the Vercel callback approach:

```bash
export TIKTOK_OAUTH_MODE="web"
export TIKTOK_REDIRECT_URI="https://YOUR-VERCEL-PROJECT.vercel.app/api/callback"
```

The redirect URI must exactly match the URI configured in TikTok Login Kit.

## Scripts

Generate auth URL:

```bash
bash docs/tiktok/tiktok_make_auth_url.sh
```

Exchange callback code for tokens:

```bash
bash docs/tiktok/tiktok_exchange_code.sh 'PASTE_CODE_FROM_CALLBACK'
```

Wait for a Vercel postbox callback code:

```bash
bash docs/tiktok/tiktok_wait_for_code.sh
```

Generate auth URL, wait for callback code, and exchange it:

```bash
bash docs/tiktok/tiktok_login_exchange.sh
```

Refresh access token:

```bash
bash docs/tiktok/tiktok_refresh_token.sh
```

Check token file status:

```bash
bash docs/tiktok/tiktok_auth_status.sh
```

Call user info:

```bash
bash docs/tiktok/tiktok_user_info.sh
```

Call authorized user's video list:

```bash
bash docs/tiktok/tiktok_video_list.sh
```

Run the full auth probe:

```bash
bash docs/tiktok/tiktok_auth_probe.sh
```

Outputs are written under:

```text
docs/tiktok/out/
```

Token file:

```text
docs/tiktok/out/tiktok_tokens.json
```

Token env snippet:

```text
docs/tiktok/out/tiktok_tokens.env
```

Full auth probe report:

```text
docs/tiktok/out/auth_probe_report.json
docs/tiktok/out/auth_probe_report.md
```

You can source this snippet after token exchange if you want token values in
your shell:

```bash
source docs/tiktok/out/tiktok_tokens.env
```

## Stitchly Workflow Fit

The first Stitchly auth test workflow should call these scripts from shell nodes
after tokens exist. OAuth login itself remains outside the workflow because it
requires browser/user interaction.

Suggested workflow:

```text
auth_status
  -> user_info_probe
  -> video_list_probe
  -> write_auth_report
```

Later, once this is stable, the API calls can become a native connector.
