import { slugify } from "./index.mjs";

addEventListener("fetch", (event) =>
  event.respondWith(
    (async () => {
      const request = event.request;
      const url = new URL(request.url);
      const redirect = await maybeRedirect(url);

      if (redirect) {
        return redirect;
      } else {
        return fetch(request);
      }
    })()
  )
);

const maybeRedirect = async (url) => {
  if (url.host !== "stadia.run") {
    return null;
  }

  const pathSegments = url.pathname.slice(1).split(/\//g);
  if (pathSegments.length === 1 && pathSegments[0] === "") {
    pathSegments.pop();
  }

  if (pathSegments.length !== 1) {
    return null;
  }

  const slug = pathSegments[0];
  if (slugify(slug) !== slug) {
    return null;
  }

  let skus;
  try {
    skus = await (
      await fetch("https://stadia.st/-/skus.json", {
        cf: {
          cacheTtlByStatus: {
            200: 1 * 60 * 60,
          },
        },
      })
    ).json();
  } catch (error) {
    console.error(error);
    return null;
  }

  const game = Object.values(skus).find(
    (sku) => sku.type === "game" && slugify(sku.name) == slug
  );

  if (!game) {
    return null;
  }

  const appId = game.app;
  return Response.redirect(`https://stadia.google.com/player/${appId}`, 301);
};
