import "./foreground/autoreload.js";
import "./foreground/jsx.js";

import { Prices } from "./foreground/data/models.js";
import { spider } from "./foreground/spider.js";

export const main = async () => {
  document.title = "stadia.observer";
  document.head.appendChild(
    Object.assign(document.createElement("link"), {
      rel: "icon",
      href: "/illufinch-violetsky-edited@2x.png",
    }),
  );

  const skus = Object.values(await (await fetch("/skus.json")).json());
  const skusById = Object.fromEntries(skus.map(sku => [sku.sku, sku]));
  const subscriptions = skus.filter(sku => sku.type === "subscription");

  const games = skus
    .filter(sku => sku.type === "game")
    .map(game =>
      Object.assign(game, {
        skus: skus.filter(sku => sku.app === game.app),
        addons: skus.filter(
          sku => sku.app === game.app && sku.type === "addon",
        ),
        bundles: skus.filter(
          sku => sku.app === game.app && sku.type === "bundle",
        ),
      }),
    );

  document.body.appendChild(<Home games={games} />);
};

const Home = ({ games }: { games: Array<any> }) => (
  <main
    style={{
      fontSize: "14px",
      maxWidth: "800px",
      margin: "8px 16px",
      backgroundImage: "url(/illufinch-violetsky-edited@2x.png)",
      backgroundPosition: "top 16px right 16px",
      backgroundRepeat: "no-repeat",
      backgroundSize: "64px",
    }}
  >
    <h1>stadia.observer</h1>

    {window.chrome?.runtime?.id && (
      <button onclick={spider}>🕷️ Spider Stadia Store</button>
    )}

    <section className="gameList">
      {games.map(game => (
        <Game {...game} />
      ))}
    </section>
  </main>
);

const Game = ({
  name,
  description,
  type,
  sku,
  app,
  addons,
  bundles,
  prices,
}) => (
  <section>
    <section
      style={{
        display: "grid",
        gridTemplateColumns: "1fr 1fr 1fr",
      }}
    >
      <h2>
        {name} [
        {type === "game" ? (
          <span>
            <a href={`http://stadia.google.com/player/${app}`}>launch game</a>{" "}
            or
          </span>
        ) : (
          type
        )}{" "}
        <a href={`https://stadia.google.com/store/details/${app}/sku/${sku}`}>
          view in store
        </a>{" "}
        {Prices.prototype.render.call(prices)}]
      </h2>
    </section>
    <section
      style={{
        display: "grid",
        gridTemplateColumns: "1fr 1fr 1fr",
      }}
    >
      <p>{description}</p>
      {addons.map(addon => (
        <section>
          <h3>
            <span style={{ fontWeight: "normal" }}>➕</span> {addon.name} [
            <a
              href={`https://stadia.google.com/store/details/${app}/sku/${addon.sku}`}
            >
              view in store
            </a>{" "}
            {Prices.prototype.render.call(addon.prices)}]
          </h3>
          <p>{description}</p>
        </section>
      ))}
    </section>
    <section
      style={{
        display: "grid",
        gridTemplateColumns: "1fr 1fr 1fr",
      }}
    >
      {bundles.map(bundle => (
        <section>
          <h3>
            <span style={{ fontWeight: "normal" }}>📦</span> {bundle.name} [
            <a
              href={`https://stadia.google.com/store/details/${app}/sku/${bundle.sku}`}
            >
              view in store
            </a>{" "}
            {Prices.prototype.render.call(bundle.prices)}]
          </h3>
          <p>{description}</p>
        </section>
      ))}
    </section>
  </section>
);
