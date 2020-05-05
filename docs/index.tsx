(async () => {
  document.title = "stadia.observer";
  document.head.appendChild(
    Object.assign(document.createElement("link"), {
      rel: "icon",
      href: "/illufinch-violetsky-edited@4x.png",
    })
  );

  type Renderable =
    | Element
    | DocumentFragment
    | Node
    | string
    | number
    | boolean
    | null
    | undefined
    | Iterable<Renderable>;

  const renderChild = (content: Renderable): Node => {
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

  const Home = ({ buttons, games }) => (
    <section>
      <h1>
        <img src="/illufinch-violetsky-edited@4x.png" />
        <slot name="title">stadia.observer</slot>
      </h1>

      <section>{buttons}</section>

      <section>
        {games.map((game) => (
          <Game {...game} />
        ))}
      </section>
    </section>
  );

  const Game = ({ name }) => (
    <section>
      <h2>{name}</h2>
    </section>
  );

  const render = (
    type: string | ((props: Record<string, Renderable>) => Element),
    props: Record<string, any>,
    ...children: Array<Renderable>
  ): Element => {
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
    <Home
      buttons={
        <button onclick={() => import("/spider.js")}>🕷️spider stadia</button>
      }
      games={games}
    />
  );
})();
