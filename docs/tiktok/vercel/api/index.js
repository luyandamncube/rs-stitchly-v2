export default function handler(req, res) {
  res.setHeader("Content-Type", "text/html; charset=utf-8");
  res.setHeader("Cache-Control", "no-store");

  return res.status(200).send(`
    <html>
      <head>
        <title>Stitchly TikTok OAuth Postbox</title>
        <meta name="viewport" content="width=device-width, initial-scale=1" />
      </head>
      <body style="font-family: system-ui, sans-serif; line-height: 1.5; max-width: 760px; margin: 40px auto; padding: 0 16px;">
        <h1>Stitchly TikTok OAuth Postbox</h1>
        <p>Use <code>/api/callback</code> as the TikTok redirect URI.</p>
        <p>Local scripts poll <code>/api/code?state=...</code> and exchange the authorization code locally.</p>
      </body>
    </html>
  `);
}
