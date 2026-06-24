import { kv } from "@vercel/kv";

const TTL_SECONDS = Number(process.env.OAUTH_CODE_TTL_SECONDS || "600");

export default async function handler(req, res) {
  const { code, state, scopes, scope, error, error_description } = req.query;

  res.setHeader("Content-Type", "text/html; charset=utf-8");
  res.setHeader("Cache-Control", "no-store");

  if (error) {
    return res.status(400).send(renderPage({
      title: "TikTok auth error",
      sections: [
        ["error", error],
        ["description", error_description || ""],
        ["state", state || ""],
      ],
      footer: "Return to your terminal/app and try the auth flow again."
    }));
  }

  if (!code || !state) {
    return res.status(400).send(renderPage({
      title: "TikTok auth callback missing code or state",
      sections: [
        ["code", code || ""],
        ["state", state || ""],
      ],
      footer: "The local polling script needs both code and state."
    }));
  }

  await kv.set(
    `tiktok-oauth:${state}`,
    {
      code,
      state,
      scopes: scopes || scope || "",
      receivedAt: new Date().toISOString(),
    },
    { ex: TTL_SECONDS }
  );

  return res.status(200).send(renderPage({
    title: "TikTok auth callback received",
    intro: "The authorization code was received. Your local script can now continue.",
    sections: [
      ["state", state || ""],
      ["scopes", scopes || scope || ""],
      ["expires_in_seconds", String(TTL_SECONDS)],
    ],
    footer: "You can close this tab."
  }));
}

function renderPage({ title, intro = "", sections = [], footer = "" }) {
  const blocks = sections.map(([label, value]) => `
    <h3>${escapeHtml(label)}</h3>
    <pre style="white-space: pre-wrap; word-break: break-word; padding: 12px; background: #f4f4f4; border-radius: 8px;">${escapeHtml(value)}</pre>
  `).join("\n");

  return `
    <html>
      <head>
        <title>${escapeHtml(title)}</title>
        <meta name="viewport" content="width=device-width, initial-scale=1" />
      </head>
      <body style="font-family: system-ui, sans-serif; line-height: 1.5; max-width: 760px; margin: 40px auto; padding: 0 16px;">
        <h1>${escapeHtml(title)}</h1>
        ${intro ? `<p>${escapeHtml(intro)}</p>` : ""}
        ${blocks}
        ${footer ? `<p>${escapeHtml(footer)}</p>` : ""}
      </body>
    </html>
  `;
}

function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&#039;");
}
