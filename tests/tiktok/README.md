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
