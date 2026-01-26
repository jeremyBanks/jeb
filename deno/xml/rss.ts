import { h, RenderableProps } from "preact";

declare global {
  namespace preact.createElement.JSX {
    interface IntrinsicElements {
      rss: preact.JSX.HTMLAttributes<HTMLElement>;
      channel: preact.JSX.HTMLAttributes<HTMLElement>;
      description: preact.JSX.HTMLAttributes<HTMLElement>;
    }
  }
}

const rfc2822DateTime = (timestamp: number | Date) => {
  const date = typeof timestamp == "number"
    ? new Date(timestamp * 1000)
    : new Date(timestamp);
  return `${
    ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"][date.getUTCDay()]
  }, ${date.getUTCDate().toString().padStart(2, "0")} ${
    [
      "Jan",
      "Feb",
      "Mar",
      "Apr",
      "May",
      "Jun",
      "Jul",
      "Aug",
      "Sep",
      "Oct",
      "Nov",
      "Dec",
    ][
      date.getUTCMonth()
    ]
  } ${date.getUTCFullYear()} ${
    date.getUTCHours().toString().padStart(2, "0")
  }:${date.getUTCMinutes().toString().padStart(2, "0")}:${
    date.getUTCSeconds().toString().padStart(2, "0")
  } UT`;
};

export const Rss = (
  { children, title, description, author, image, link, self, next, prev, type }:
    RenderableProps<{
      title?: string;
      description?: string;
      language?: string;
      link?: string;
      self?: string;
      next?: string;
      prev?: string;
      author?: string;
      image?: string;
      type?: "serial" | "episodic";
    }>,
) =>
  h(
    "rss" as any,
    {
      "version": "2.0",
      "xmlns:content": "http://purl.org/rss/1.0/modules/content/",
      "xmlns:itunes": "http://www.itunes.com/dtds/podcast-1.0.dtd",
      "xmlns:atom": "http://www.w3.org/2005/Atom",
    },
    h(
      "channel",
      {},
      title && h("title", {}, title),
      description && h("description", {}, description),
      link && h("link", {}, link),
      self &&
        h("atom:link", {
          rel: "self",
          href: self,
          type: "application/rss+xml",
        }),
      next &&
        h("atom:link", {
          rel: "next",
          href: next,
          type: "application/rss+xml",
        }),
      prev &&
        h("atom:link", {
          rel: "prev",
          href: prev,
          type: "application/rss+xml",
        }),
      author && h("itunes:author", {}, author),
      type && h("itunes:type", {}, type),
      image && h("itunes:image", { href: image.toString() }),
      children,
    ),
  );

export const Item = (
  { children, title, link, guid, pubDate, enclosure, duration }:
    RenderableProps<{
      title?: string;
      link?: string;
      guid?: string;
      pubDate?: number;
      duration?: number;
      enclosure?: {
        url: string;
        type: string;
        length?: number;
      };
    }>,
) => {
  return h(
    "item",
    {},
    title && h("title", {}, title),
    link && h("link", {}, link),
    guid && h("guid", {}, guid),
    pubDate && h("pubDate", {}, rfc2822DateTime(pubDate)),
    enclosure && h("enclosure", enclosure),
    duration && h("itunes:duration", {}, duration),
    children && h("description", {}, children),
  );
};
