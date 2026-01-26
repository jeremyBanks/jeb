import render from "preact-render-to-string";
import { VNode } from "preact";
import { Rss } from "~/xml/rss.ts";

export const renderXml = (node: VNode, opts?: {
  pretty?: boolean;
  type?: string;
}) =>
  new Response(
    `<?xml version="1.0" encoding="UTF-8"?>\n` +
      render(node, null, { xml: true, pretty: opts?.pretty ? "  " : " " }),
    {
      headers: {
        "Content-Type": opts?.type ?? node.type != Rss
          ? "application/xml"
          : "application/rss+xml",
      },
    },
  );
