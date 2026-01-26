import { MiddlewareHandler } from "$fresh/server.ts";

export const handler: MiddlewareHandler = async (request, context) => {
  const url = new URL(request.url);

  if (url.hostname.startsWith("fic.") && url.protocol !== "https:") {
    url.protocol = "https:";
    return new Response(null, {
      status: 308,
      headers: {
        Location: url.toString(),
        "Strict-Transport-Security": url.hostname === "fic.is"
          ? "max-age=63072000; includeSubDomains; preload"
          : "max-age=14400",
      },
    });
  }

  const response = await context.next();

  if (url.hostname === "fic.is") {
    response.headers.set(
      "Strict-Transport-Security",
      "max-age=63072000; includeSubDomains; preload",
    );
  }

  if (
    !response.headers.has("Cache-Control") &&
    response.headers.get("Content-Type") === "text/html"
  ) {
    response.headers.append(
      "Cache-Control",
      [
        `max-age=${60 * 60 * 24 * 32}`,
        `stale-while-revalidate=${60 * 60 * 24 * 256}`,
        `stale-if-error=${60 * 60 * 24 * 256}`,
      ].join(", "),
    );
  }

  return response;
};
