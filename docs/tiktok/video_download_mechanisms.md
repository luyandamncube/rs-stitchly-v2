# TikTok Video Download Mechanism Review

This note captures the current ingestion options for TikTok video archive work.
It assumes the OAuth bootstrap path is the repo-local Stitchly workflow
`tiktok_auth_bootstrap`, followed by `tiktok_auth_probe`, using the deployed
Vercel callback postbox and Upstash Redis code store.

## Current Auth Baseline

- `tiktok_auth_bootstrap` generates the Login Kit URL, waits for the Vercel
  callback code, exchanges it, and stores tokens under
  `.stitchly/secrets/tiktok/`.
- `tiktok_auth_probe` validates the saved token, calls `/v2/user/info/`, and
  calls `/v2/video/list/`.
- The Display API token is useful for the authorized user's own public videos.
  It does not grant arbitrary URL download access.

## Mechanism Matrix

| Mechanism | Test | Archive fit | Exclude if |
| --- | --- | --- | --- |
| `yt-dlp` metadata only | `yt-dlp --dump-json <url>` via `tests/tiktok/probe_ytdlp_metadata.sh` | Candidate for public URL metadata discovery only. | Cannot resolve metadata reliably for normal public URLs. |
| `yt-dlp` download | `ENABLE_MEDIA_DOWNLOAD=true bash docs/tiktok/tiktok_download_video.sh <url>` for a single URL, or `ENABLE_MEDIA_DOWNLOAD=true bash tests/tiktok/run_probe.sh` for harness URLs. The single-URL script defaults to H.264 MP4 when available. | Candidate only for URLs we have permission to archive and only after explicit opt-in. | Requires login/cookies for most target URLs, is unstable, returns audio-only, or only exposes codecs that the target player cannot decode. |
| TikTok oEmbed | `tests/tiktok/probe_oembed.sh` calls `https://www.tiktok.com/oembed?url=...`. | Useful as a public URL reachability/embed-card probe. | Only returns embed/card metadata and is not enough for the archive workflow. |
| Display API | `docs/tiktok/tiktok_video_list.sh` or `tests/tiktok/probe_display_api.sh` calls `/v2/video/list/` with OAuth token. | Strong candidate for "archive my own authorized TikTok videos" metadata and embed links. | Requires per-user OAuth and only returns the authorized user's videos; exclude for arbitrary URLs. |
| Research API | `tests/tiktok/probe_research_api.sh` with an approved research token. | Metadata-only candidate for approved research use cases with query/date constraints. | Access is unavailable, approval is too heavy, or returned metadata is insufficient. |
| Data Portability API | Keep as a non-mutating eligibility check until a real own-account export workflow exists. | Candidate only for explicit "export my own TikTok account data" flows. | Exclude for generic URL ingestion or any workflow that should not create account export requests. |
| SaaS downloader API | Vendor-specific smoke call after a vendor is selected. | Candidate only if legal/compliance posture, pricing, and output reproducibility are clear. | Exclude until those vendor criteria are documented and the smoke call is reproducible. |

## Recommended Path

1. Treat the new OAuth workflows as the canonical auth path.
2. Use Display API for authorized-user inventory and metadata.
3. Use oEmbed and `yt-dlp --dump-json` as non-authoritative public URL probes.
4. Keep media download disabled by default and require
   `ENABLE_MEDIA_DOWNLOAD=true` for any download trial.
5. Do not build arbitrary TikTok URL downloading as the primary workflow unless
   the probe report shows stable, permitted, reproducible results.

## Multi-Video Pipeline

The saved Stitchly pipeline `tiktok_video_download_probe` accepts a list of
TikTok URLs in its first node, `Video Download Config`. Add one row per video in
the `values` block:

```sql
from (
  values
    ('https://www.tiktok.com/@example/video/1111111111111111111'),
    ('https://www.tiktok.com/@example/video/2222222222222222222')
) as urls(video_url)
```

The pipeline downloads each row with `docs/tiktok/tiktok_download_video.sh`,
parses one JSON result per video, logs a table of results, and writes the full
manifest to Parquet.

## Verification Commands

```bash
bash docs/tiktok/tiktok_auth_probe.sh
bash docs/tiktok/tiktok_download_video.sh '<tiktok-url>'
ENABLE_MEDIA_DOWNLOAD=true bash docs/tiktok/tiktok_download_video.sh '<tiktok-url>'
bash tests/tiktok/run_probe.sh
ENABLE_MEDIA_DOWNLOAD=true bash tests/tiktok/run_probe.sh
```
