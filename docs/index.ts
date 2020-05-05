(async () => {
  document.title = "stadia.observer";
  document.head.appendChild(
    Object.assign(document.createElement("link"), {
      rel: "icon",
      href: "/illufinch-violetsky-edited@4x.png",
    })
  );

  const templates = Object.fromEntries(
    [
      ...document.querySelectorAll("template[data-key]"),
    ].map((el: HTMLTemplateElement) => [el.dataset.key, el.content])
  );

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
