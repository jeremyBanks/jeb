import { render } from "./render.js";

document.title = "stadia.observer";
document.head.appendChild(
  Object.assign(document.createElement("link"), {
    rel: "icon",
    href: "/illufinch-violetsky-edited@2x.png",
  })
);

const Home = ({ buttons, games }) => (
  <section>
    <h1>stadia.observer</h1>

    {window.chrome?.runtime?.id && (
      <button onclick={async () => (await import("./spider.js")).spider()}>
        🕷️spider stadia
      </button>
    )}
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

(async () => {
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

  document.body.appendChild(<Home games={games} />);
})();
