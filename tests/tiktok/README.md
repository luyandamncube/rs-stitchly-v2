# TikTok Mechanism Probe Harness

This harness tests which TikTok ingestion mechanisms are viable before we build
a Stitchly workflow around them.

It is metadata-first. It does not download media unless explicitly enabled.

## Setup

```bash
cp tests/tiktok/.env.example tests/tiktok/.env
cp tests/tiktok/input_urls.example.txt tests/tiktok/input_urls.txt
```

Add representative URLs to `tests/tiktok/input_urls.txt`.

Optional tools:

- `yt-dlp` for public URL metadata and optional media download probes.
- `curl` for API and oEmbed probes.
- `python3` for URL encoding and report generation.

## OAuth Setup

If you only have the TikTok developer app client key/secret, first generate a
user access token through Login Kit OAuth.

Fill these values in `tests/tiktok/.env`:

```bash
TIKTOK_CLIENT_KEY=
TIKTOK_CLIENT_SECRET=
TIKTOK_REDIRECT_URI=https://example.com/tiktok/callback
TIKTOK_SCOPES=user.info.basic,video.list
```

`TIKTOK_REDIRECT_URI` must exactly match a redirect URI configured in the
TikTok developer app's Login Kit product. TikTok's Web Login docs require HTTPS
redirect URIs for web apps, so use your configured HTTPS callback or a temporary
HTTPS tunnel if needed.

Generate the authorization URL:

```bash
bash tests/tiktok/oauth_start.sh
```

Open the printed URL in a browser. After approval, TikTok redirects to your
callback URL with a `code=...` query parameter. Copy that code into `.env`:

```bash
TIKTOK_AUTH_CODE=...
```

Exchange the authorization code for tokens:

```bash
bash tests/tiktok/oauth_exchange.sh
```

The raw response is written to:

```text
tests/tiktok/out/oauth_token_response.json
```

The script also writes a local token snippet to:

```text
tests/tiktok/out/oauth_tokens.env
```

Copy the returned `TIKTOK_ACCESS_TOKEN` into `tests/tiktok/.env`, then run the
main probe.

## Run

```bash
bash tests/tiktok/run_probe.sh
```

Outputs:

```text
tests/tiktok/out/
  mechanism_report.json
  mechanism_report.md
  yt_dlp_metadata.jsonl
  yt_dlp_download.jsonl
  oembed.jsonl
  display_api_response.json
  research_api_response.json
  data_portability.jsonl
  errors.jsonl
```

## Safe Download Probe

Downloads are disabled by default.

To test downloads for URLs you have permission to archive:

```bash
ENABLE_MEDIA_DOWNLOAD=true bash tests/tiktok/run_probe.sh
```

Downloaded files go under `tests/tiktok/out/downloads/`.

## Decision Rule

Use this harness to exclude mechanisms that are unavailable, metadata-only,
approval-heavy, or unreliable for representative URLs.
