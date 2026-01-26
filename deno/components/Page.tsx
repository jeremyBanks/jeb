import { PageProps } from "$fresh/server.ts";
import { Component, RenderableProps } from "preact";
import { asset, Head } from "$fresh/runtime.ts";
import { Fragment, h } from "preact";

export type Props = {
  _?: unknown;
};

export default (props: RenderableProps<Props>) => (
  <>
    <Head>
    </Head>

    <main class="flex flex-grow self-stretch items-start justify-center bg-yellow-50 color-white max-w-full overflow-y-scroll">
      {props.children}
    </main>
  </>
);
