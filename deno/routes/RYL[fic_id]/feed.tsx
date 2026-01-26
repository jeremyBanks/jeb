import {
  HandlerContext,
  Handlers,
  PageProps,
  RenderContext,
} from "$fresh/server.ts";

export const config = {
  routeOverride: "/:fic_id(RYL[0-9A-Z]{7})(.rss|/feed.xml)",
};

import * as fakeDom from "deno-dom";
import { IS_BROWSER } from "$fresh/runtime.ts";
import * as z from "zod";
import { renderXml } from "~/xml/xml.ts";
import { Item, Rss } from "~/xml/rss.ts";
import { load } from "~/utils/data.ts";
import { h } from "preact";

const { DOMParser } = IS_BROWSER
  ? globalThis
  : fakeDom as unknown as typeof globalThis;

const Spine = z.object({
  id10: z.string(),
  title: z.string(),
  length: z.number(),
  chapters: z.array(z.object({
    id10: z.string(),
    timestamp: z.number(),
    title: z.string(),
    length: z.number(),
    starts_with: z.string(),
  })),
});
type Spine = z.infer<typeof Spine>;

export const handler: Handlers = {
  async GET(request, context) {
    const url = new URL(request.url);

    const spine = Spine.parse(
      await load(`spines/${context.params.fic_id}`),
    );

    const pageSize = 32;

    const pageCount = Math.ceil(spine.chapters.length / pageSize);
    const page = parseInt(url.searchParams.get("page") ?? "1", 10);
    if (!Number.isFinite(page) || page < 1 || page > pageCount) {
      return new Response("invalid page number", { status: 400 });
    }
    const offset = (page - 1) * pageSize;
    const chapters = spine.chapters.slice(offset, offset + pageSize);

    const ficUrl = `https://${url.host}/${context.params.fic_id}`;

    const first = `${ficUrl}/feed.xml`;
    const self = `${first}${page > 1 ? `?page=${page}` : ""}`;
    const next = page < pageCount ? `${first}?page=${page + 1}` : undefined;
    const prev = page > 1
      ? page == 2 ? first : `${first}?page=${page - 1}`
      : undefined;

    return renderXml(
      <Rss
        title={spine.title}
        link={ficUrl}
        self={self}
        prev={prev}
        next={next}
        image={`${ficUrl}/cover.png`}
        description={[...spine.title].sort(() => Math.random() - 0.125).join(
          "",
        )}
        author={[...spine.title].sort(() => Math.random() - 0.125).join("")}
        type="serial"
      >
        {chapters.map((chapter) => (
          <Item
            pubDate={chapter.timestamp}
            title={chapter.title}
            link={`${ficUrl}/${chapter.id10}`}
            guid={`${ficUrl}/${chapter.id10}`}
            enclosure={{
              type: "audio/ogg",
              url: `${ficUrl}/${chapter.id10}.opus`,
            }}
          >
            {chapter.starts_with}
          </Item>
        ))}
      </Rss>,
    );
  },
};
