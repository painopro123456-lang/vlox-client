export default {
  async fetch(request, env) {
    const origin = request.headers.get("Origin") || "*";
    const cors = {
      "Access-Control-Allow-Origin": "*",
      "Access-Control-Allow-Methods": "GET, OPTIONS",
      "Access-Control-Allow-Headers": "Content-Type",
      "Access-Control-Max-Age": "86400",
      "Vary": "Origin"
    };
    const respond = (body, status = 200, headers = {}) =>
      new Response(body, { status, headers: { ...cors, ...headers } });
    if (request.method === "OPTIONS") return respond(null, 204);
    if (request.method !== "GET") return respond("Method not allowed", 405);
    if (!env.CURSEFORGE_API_KEY) return respond("CurseForge service is not configured", 503);
    const url = new URL(request.url);
    const upstream = new URL("https://api.curseforge.com/v1" + url.pathname);
    const headers = { "x-api-key": env.CURSEFORGE_API_KEY, "Accept": "application/json" };
    if (url.pathname === "/v1/mods/search") {
      const query = (url.searchParams.get("searchFilter") || "").slice(0, 100);
      upstream.searchParams.set("gameId", "432");
      upstream.searchParams.set("classId", "4471");
      upstream.searchParams.set("searchFilter", query);
      upstream.searchParams.set("pageSize", "20");
      upstream.searchParams.set("sortField", "6");
      upstream.searchParams.set("sortOrder", "desc");
      upstream.searchParams.set("gameVersion", "26.2");
      upstream.searchParams.set("modLoaderType", "4");
    } else {
      const filesList = url.pathname.match(/^\/v1\/mods\/(\d+)\/files$/);
      const fileDetails = url.pathname.match(/^\/v1\/mods\/(\d+)\/files\/(\d+)$/);
      if (!filesList && !fileDetails) return respond("Not found", 404);
      if (filesList) {
        upstream.searchParams.set("gameVersion", "26.2");
        upstream.searchParams.set("modLoaderType", "4");
        upstream.searchParams.set("pageSize", "20");
        upstream.searchParams.set("index", "0");
      }
    }
    try {
      const upstreamResponse = await fetch(upstream.toString(), { headers });
      const body = await upstreamResponse.text();
      return respond(body, upstreamResponse.status, {
        "Content-Type": upstreamResponse.headers.get("Content-Type") || "application/json",
        "Cache-Control": "public, max-age=60"
      });
    } catch {
      return respond("CurseForge service request failed", 502);
    }
  }
};
