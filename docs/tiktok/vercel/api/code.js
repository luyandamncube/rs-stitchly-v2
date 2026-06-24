import { kv } from "@vercel/kv";

export default async function handler(req, res) {
  res.setHeader("Cache-Control", "no-store");

  const { state, consume } = req.query;
  if (!state) {
    return res.status(400).json({ ok: false, error: "missing_state" });
  }

  const key = `tiktok-oauth:${state}`;
  const payload = await kv.get(key);
  if (!payload) {
    return res.status(404).json({ ok: false, error: "not_found" });
  }

  if (consume !== "false") {
    await kv.del(key);
  }

  return res.status(200).json({
    ok: true,
    code: payload.code,
    state: payload.state,
    scopes: payload.scopes || "",
    receivedAt: payload.receivedAt || null,
  });
}
