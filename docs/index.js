"use strict";
(async () => {
  document.title = "stadia.observer";
  document.head.appendChild(
    Object.assign(document.createElement("link"), {
      rel: "icon",
      href: "/illufinch-violetsky-edited@4x.png",
    })
  );
  const renderChild = (content) => {
    if (content === null || content === undefined) {
      return document.createElement("span");
    } else if (content instanceof Node) {
      return content;
    } else if (
      typeof content === "string" ||
      typeof content === "number" ||
      typeof content === "boolean"
    ) {
      return document.createTextNode(String(content));
    } else {
      const container = document.createDocumentFragment();
      for (const child of content) {
        container.appendChild(renderChild(child));
      }
      return container;
    }
  };
  const Home = ({ buttons, games }) =>
    render(
      "section",
      null,
      render(
        "h1",
        null,
        render("img", { src: "/illufinch-violetsky-edited@4x.png" }),
        render("slot", { name: "title" }, "stadia.observer")
      ),
      render("section", null, buttons),
      render(
        "section",
        null,
        games.map((game) => render(Game, Object.assign({}, game)))
      )
    );
  const Game = ({ name }) => render("section", null, render("h2", null, name));
  const render = (type, props, ...children) => {
    if (typeof type === "string") {
      const el = document.createElement(type);
      Object.assign(el, props);
      el.appendChild(renderChild(children));
      return el;
    } else {
      return type(Object.assign(props, { children }));
    }
  };
  const skus = Object.values(await (await fetch("/skus.json")).json());
  const skusById = Object.fromEntries(skus.map((sku) => [sku.sku, sku]));
  const subscriptions = skus.filter((sku) => sku.type === "subscription");
  const games = skus
    .filter((sku) => sku.type === "game")
    .map((game) =>
      Object.assign(game, {
        skus: skus.filter((sku) => sku.app === game.app),
        addons: skus.filter(
          (sku) => sku.app === game.app && sku.type === "addon"
        ),
        bundles: skus.filter(
          (sku) => sku.app === game.app && sku.type === "bundle"
        ),
      })
    );
  document.body.appendChild(
    render(Home, {
      buttons: render(
        "button",
        { onclick: () => import("/spider.js") },
        "\uD83D\uDD77\uFE0Fspider stadia"
      ),
      games: games,
    })
  );
})();
//# sourceMappingURL=index.js.map
