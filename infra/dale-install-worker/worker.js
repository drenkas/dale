// Serves install.sh from the dale repo at borkiss.net/dale-install.sh.
// The script always tracks main, so releases need no worker redeploys.
export default {
  async fetch() {
    const upstream = await fetch(
      "https://raw.githubusercontent.com/lubluniky/dale/main/install.sh",
      { cf: { cacheTtl: 300, cacheEverything: true } },
    );
    if (!upstream.ok) {
      return new Response("# dale install script temporarily unavailable\n", {
        status: 503,
        headers: { "content-type": "text/x-shellscript; charset=utf-8" },
      });
    }
    return new Response(upstream.body, {
      headers: {
        "content-type": "text/x-shellscript; charset=utf-8",
        "cache-control": "public, max-age=300",
      },
    });
  },
};
