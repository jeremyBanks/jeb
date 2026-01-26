import { Handlers, PageProps, RenderContext } from "$fresh/server.ts";
import { z } from "zod";
import { Head, IS_BROWSER } from "$fresh/runtime.ts";
import { h } from "preact";
import { apply, css, tw } from "twind/css";
import Page from "~/components/Page.tsx";
import * as fakeDom from "deno-dom";
import { load } from "~/utils/data.ts";

const { DOMParser } = IS_BROWSER
  ? globalThis
  : fakeDom as unknown as typeof globalThis;
export const config = {
  routeOverride: "/:fic_id(RYL[0-9A-Z]{7})/:chapter_id(C[0-9A-Z]{9})",
};

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

const Chapter = z.object({
  id10: z.string(),
  timestamp: z.number(),
  title: z.string(),
  html: z.string(),
});
type Chapter = z.infer<typeof Chapter>;

export const handler: Handlers = {
  async GET(_request, context) {
    const spine = Spine.parse(await load(`spines/${context.params.fic_id}`));

    const chapter = Chapter.parse(
      await load(
        `chapters/${context.params.fic_id}${context.params.chapter_id}`,
      ),
    );

    const next =
      spine.chapters.filter((c) => c.timestamp > chapter.timestamp)[0];
    const previous = spine.chapters.filter((c) =>
      c.timestamp < chapter.timestamp
    ).pop();

    return await context.render({ chapter, spine, next, previous });
  },
};

export default (
  { data: { spine, chapter, next, previous } }: PageProps<
    {
      spine: Spine;
      chapter: Chapter;
      next: Spine["chapters"][0];
      previous: Spine["chapters"][0];
    }
  >,
) => {
  const nav = (
    <>
      <nav class="flex text-xl">
        {previous
          ? (
            <a
              href={`/${spine.id10}/${previous.id10}`}
              class="block center flex-grow bg-red-50 text-center p-4"
              rel="previous"
            >
              Previous
            </a>
          )
          : null}
        {next
          ? (
            <a
              href={`/${spine.id10}/${next.id10}`}
              class="block flex-grow bg-green-300 text-center p-4"
              rel="next"
            >
              Next
            </a>
          )
          : null}
      </nav>
    </>
  );
  return (
    <Page>
      <Head>
        <title>{chapter.title} &mdash; {spine.title}</title>
        <meta name="robots" content="noindex" />
      </Head>
      <main class="p-10 text-lg bg-white lg:w-192">
        <nav class="flex">
          <a
            href={`/${spine.id10}`}
            class="block flex-grow bg-yellow-100 text-center font-bold p-4"
          >
            {spine.title}
          </a>
        </nav>
        <h1 class="text-xl font-bold mt-4 p-4 text-center text-xl">
          {chapter.title}
        </h1>
        {nav}
        <div
          class={tw`text-lg p-4 ${
            css({
              "& p": css`text-indent: .5rem; scroll-snap-align: top; ${
                apply("my-2")
              }`,
            })
          }`}
          dangerouslySetInnerHTML={{ __html: chapter.html }}
        />
        {nav}
      </main>
    </Page>
  );
};
