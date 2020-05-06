import { render } from "./render.js";

document.title = "stadia.observer";
document.head.appendChild(
  Object.assign(document.createElement("link"), {
    rel: "icon",
    href: "/illufinch-violetsky-edited@2x.png",
  })
);

const Home = ({ games }: any) => (
  <main
    style={`
    font-size: 14px;
    max-width: 800px;
    margin: 8px 16px;

    background-image: url(/illufinch-violetsky-edited@2x.png);
    background-position: top 16px right 16px;
    background-repeat: no-repeat;
    background-size: 64px;
  `}
  >
    <h1>stadia.observer</h1>

    {window.chrome?.runtime?.id && (
      <button onclick={async () => (await import("./spider.js")).spider()}>
        🕷️spider stadia
      </button>
    )}

    <section>
      {games.map((game) => (
        <Game {...game} />
      ))}
    </section>
  </main>
);

const Game = ({ name, description, type, sku, app }) => (
  <section>
    <h2>{name}</h2>
    <p>
      {(type === "game") ? <a href={`http://stadia.google.com/player/${app}`}>launch game</a> : type}
      {" "}
      <a href={`https://stadia.google.com/store/details/${app}/sku/${sku}`}>store</a>
    </p>
    {" "}
    <p>{description}</p>
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
