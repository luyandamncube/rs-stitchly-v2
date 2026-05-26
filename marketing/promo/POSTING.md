# Posting copy for the promo

## The video

- `out/duckle-promo.mp4` - 30 s, 1080p, silent, 1.2 MB
- `out/duckle-promo-thumbnail.jpg` - end-card freeze frame for use as YouTube thumbnail or OG image

Silent video works well on Reddit, Twitter/X, LinkedIn, and most subreddits with auto-play. For YouTube you may want to add a music bed; the file `out/duckle-promo.mp4` has no audio track so you can drop one in without conflict.

## YouTube

**Title (under 70 chars):**
Duckle: open-source ETL that runs locally, in a 30 MB binary

**Description:**
```
Duckle is a local-first, open-source ETL / ELT / streaming platform. Drag-drop
pipelines on a canvas, native DuckDB SQL execution, in-app Git for GitHub and
GitLab, and Duckie - a local AI assistant that turns natural language into
runnable pipelines without ever sending your data anywhere.

200+ components. Single 30 MB executable. No JVM, no license, no cloud.

Download: https://github.com/SouravRoy-ETL/duckle/releases/latest
Source:   https://github.com/SouravRoy-ETL/duckle

Chapters
0:00  Intro
0:04  Canvas
0:09  Duckie AI Assistant
0:14  Component library
0:19  In-app Git + CI
0:24  Free, open source, 30 MB
```

**Tags:** etl, elt, data engineering, duckdb, open source, rust, tauri, react, local first, ai, llama cpp, qwen, pipelines, talend alternative, informatica alternative, matillion alternative

**Thumbnail:** `out/duckle-promo-thumbnail.jpg`

## Reddit

### r/dataengineering
**Title:** I built Duckle - an open-source, local-first ETL platform with a drag-drop UI, DuckDB execution, and a local AI assistant. 30 MB binary, no JVM.

**Body:**
```
Hey r/dataengineering,

After getting tired of Talend's JVM dance and the heavyweight feel of every
"modern" data platform that ends up being a SaaS, I built [Duckle][1] - a
local-first ETL/ELT/streaming tool that runs as a single 30 MB executable.

What it does
- Drag-drop pipeline canvas (sources, transforms, sinks, control flow)
- Compiles pipelines to native DuckDB SQL and runs them locally
- 200+ components including REST, S3, Postgres, MySQL, MongoDB, Kafka, NATS,
  RabbitMQ, GCP Pub/Sub, Kinesis, DynamoDB, FTP/SFTP, SOAP, OData, and more
- In-app Git for both GitHub and GitLab (commit / push / pull / branch) with a
  pipeline-status badge in the topbar
- Duckie - a local AI assistant that turns "load orders.csv, drop rows where
  amount is null, write parquet" into an actual pipeline you can insert on the
  canvas. Runs llama.cpp + Qwen2.5-Coder-1.5B on your CPU. No API key, no data
  leaves the machine.

What it isn't
- Not a distributed runner. Single-node, DuckDB-backed. If you need 50-node
  shuffles, this is not for you.
- The streaming connectors are batch-mode for now (Pulsar still TODO).
- Apache iceberg / Delta tables are roadmap, not present.

It's MIT/Apache-2 dual licensed, the whole thing is in the open at
github.com/SouravRoy-ETL/duckle. Binaries for Windows, Linux, and macOS
(Apple Silicon) on the releases page.

30-second silent demo: <attach duckle-promo.mp4>

Feedback welcome - especially on the connector list (which one is missing
that you'd actually use) and on the AI assistant UX.

[1]: https://github.com/SouravRoy-ETL/duckle
```

### r/rust
**Title:** Duckle - I shipped a local-first ETL platform in Rust (Tauri + DuckDB + llama.cpp). Single 30 MB binary, opens in under a second.

**Body:**
```
Cross-posting from r/dataengineering because the stack might interest folks
here.

Duckle is an open-source ETL/ELT/streaming tool I've been building. Rust
backend, React frontend, Tauri 2 shell, DuckDB CLI as the SQL engine, and
llama.cpp + Qwen2.5-Coder-1.5B for the local AI assistant.

Rust bits worth calling out
- Pure-Rust AWS SigV4 for DynamoDB and Kinesis (no aws-sdk-rust dependency)
- boa_engine for the JS code blocks
- wasmi for the WASM code blocks
- imap 3.x alpha + lettre for email source/sink
- Custom XML walker built on quick-xml that handles SOAP namespacing
- llama-server subprocess managed from Rust with an OpenAI-compatible HTTP
  shim, lazy-spawned on first chat message

Single 30 MB binary on Windows/Linux/macOS. Opens in well under a second
(no JVM cold start). MIT/Apache-2 dual licensed.

github.com/SouravRoy-ETL/duckle

Happy to talk about any of the implementation details - the Tauri custom
protocol bug I hit on first release was particularly painful.

30 s silent demo: <attach duckle-promo.mp4>
```

### r/opensource
**Title:** Duckle - open-source ETL platform with drag-drop UI, local execution, and a local AI assistant. Single 30 MB binary, MIT/Apache-2.

**Body:**
```
Hey r/opensource,

I've been building [Duckle][1] in the open for the last few weeks - an open
source ETL/ELT/streaming platform that runs locally as a single 30 MB
executable. MIT/Apache-2 dual licensed.

The pitch in one line: the connector depth of Talend without the JVM dance.

Key bits
- Drag-drop pipeline canvas, 200+ components
- DuckDB-backed native SQL execution
- In-app Git integration (GitHub + GitLab) with CI status in the topbar
- Duckie - local AI assistant via llama.cpp, no API key required
- Binaries for Windows / Linux / macOS (Apple Silicon)

Roadmap is in the open at github.com/SouravRoy-ETL/duckle, contributions
welcome - the easiest places to start are: adding a SaaS REST alias, writing
an integration test for one of the existing connectors, or improving the
component reference docs.

30 s silent demo: <attach duckle-promo.mp4>

[1]: https://github.com/SouravRoy-ETL/duckle
```

### r/selfhosted (lighter angle)
**Title:** Duckle - self-hostable, single-binary ETL platform with a built-in AI assistant that runs entirely on your CPU.

**Body:**
```
Posting here because it's local-first by default. Duckle is an open source
ETL/ELT tool packaged as a single 30 MB executable. No cloud account, no
license keys, no SaaS layer. Reads from / writes to whatever you point it at:
local files, S3-compatible storage, your own Postgres / MySQL / MongoDB,
Kafka, NATS, RabbitMQ.

The AI assistant ("Duckie") is also local - llama.cpp running Qwen2.5-Coder-1.5B
on your CPU. No data leaves the machine, no API key needed. You type what you
want the pipeline to do, it generates a runnable pipeline, you click Insert
and it appears on the canvas.

MIT/Apache-2, source at github.com/SouravRoy-ETL/duckle.

30 s silent demo attached.
```

## Twitter / X

**Tweet 1 (with video attached):**
```
I built Duckle - an open-source ETL platform that runs locally as a single
30 MB binary.

  - drag-drop pipeline canvas
  - 200+ components
  - in-app git + CI for GitHub & GitLab
  - Duckie: a local AI assistant (no API key, runs on your CPU)

github.com/SouravRoy-ETL/duckle
```

**Reply thread:**
```
2/  No JVM. No cloud. No license keys. Opens in well under a second on Windows,
    Linux, and macOS (Apple Silicon).

3/  Stack: Rust + Tauri 2 + React 19. Native DuckDB CLI for SQL execution,
    llama.cpp + Qwen2.5-Coder-1.5B for the AI assistant. Pure-Rust AWS SigV4
    for DynamoDB and Kinesis (no AWS SDK dependency).

4/  MIT/Apache-2 dual licensed. Roadmap is public.
    Issues, PRs, and "I'd actually use Duckle if it had X" comments all welcome.
```

## LinkedIn

```
I shipped Duckle today - an open-source ETL/ELT/streaming platform that runs
locally as a single 30 MB executable.

The motivation: I'd been working with Talend for years and was tired of the
JVM startup cost, the heavyweight Studio installs, and the license-key dance.
Duckle is the same connector depth in a fraction of the footprint, with a
drag-drop canvas, in-app Git integration for GitHub and GitLab, and a local
AI assistant (no API key, runs on your CPU via llama.cpp) that turns natural
language into runnable pipelines.

MIT/Apache-2. Binaries for Windows, Linux, and macOS on the releases page.
Source at github.com/SouravRoy-ETL/duckle.

#dataengineering #etl #opensource #duckdb #rust
```

## Hacker News (Show HN)

**Title:**
Show HN: Duckle - open-source local-first ETL platform in a 30 MB binary

**Body (top comment):**
```
Hi HN,

Duckle is an open-source ETL/ELT/streaming platform I've been building. It
runs as a single 30 MB executable on Windows, Linux, and macOS (Apple Silicon).

The pitch: Talend's connector depth without the JVM dance. Drag-drop pipeline
canvas, 200+ components (sources, transforms, sinks, control flow, code
blocks, AI), native DuckDB SQL execution, in-app Git for GitHub and GitLab,
and a local AI assistant that turns natural language into runnable pipelines
via llama.cpp + Qwen2.5-Coder-1.5B. No API keys, no data leaves the machine.

Stack: Rust backend, React frontend, Tauri 2 shell. A few things I'm proud of:
pure-Rust AWS SigV4 for DynamoDB and Kinesis (no aws-sdk dependency), an
XML walker on top of quick-xml that handles SOAP namespacing, a custom
protocol fix for embedded vs dev frontend that bit me on v0.0.7.

What it isn't: distributed. Single-node, DuckDB-backed. If you need 50-node
shuffles, this is not the right tool.

MIT/Apache-2 dual licensed. Roadmap is public. Happy to answer questions
about any of the design decisions.

github.com/SouravRoy-ETL/duckle
```

## Notes on posting cadence

- Post Reddit posts on different days so they don't look coordinated
- r/dataengineering, r/rust, and HN are the highest-signal communities for
  this audience
- LinkedIn and Twitter posts can go up the same day as the YouTube upload
- If a post gains traction, engage in the comments within the first 2-3
  hours - that's when Reddit/HN ranking is most sensitive to comment velocity
