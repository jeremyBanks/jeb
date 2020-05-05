"use strict";
(async () => {
  fetch("https://stadia.google.com/home");
  const skus = Object.values(await (await fetch("../skus.json")).json());
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
  for (const game of games) {
    document.body.appendChild(
      Object.assign(document.createElement("p"), {
        textContent: game.name + ":" + game.addons.map((x) => x.name).join(" "),
      })
    );
  }
  // console.log(games);
  // console.log(sku.content);
  // const root = document.body.attachShadow({mode: 'open'});
  // const el = sku.content.cloneNode(true);
  // console.log({el});
  // root.appendChild(el);
  // console.log(root);
  // console.log({el});
})();
//# sourceMappingURL=page.js.map
