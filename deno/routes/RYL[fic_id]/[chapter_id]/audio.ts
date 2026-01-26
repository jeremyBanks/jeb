import { HandlerContext } from "$fresh/server.ts";
export const config = {
  routeOverride:
    "/:fic_id(RYL[0-9A-Z]{7})/:chapter_id(C[0-9A-Z]{9})\.(opus|ogg|mp3|m4a|aac)",
};

export const handler = (
  _request: Request,
  { params: { fic_id, chapter_id } }: HandlerContext,
) =>
  new Response(
    null,
    {
      status: 302,
      headers: {
        "Location": "https://s3.amazonaws.com/sfic/0.ogg",
        "Cache-Control": [
          `max-age=${24 * 32}`,
          `stale-while-revalidate=${24 * 256}`,
          `stale-if-error=${24 * 256}`,
        ].join(", "),
      } ?? {
        "Location":
          `https://s3.amazonaws.com/sfic/audio/${fic_id}/${chapter_id}.ogg`,
        "Cache-Control": [
          `max-age=${60 * 60 * 24 * 32}`,
          `stale-while-revalidate=${60 * 60 * 24 * 256}`,
          `stale-if-error=${60 * 60 * 24 * 256}`,
        ].join(", "),
      },
    },
  );
