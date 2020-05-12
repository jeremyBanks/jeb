import * as models from "./data/models.js";

import { spider } from "./spider.js";

export const Home: JSX.FC<{ skus: Array<models.Sku> }> = ({ skus }) => (
  <main>
    {browser?.runtime?.id && (
      <button
        onclick={spider}
        style={{
          cursor: "pointer",
          position: "absolute",
          right: "16px",
          top: "16px",
        }}
      >
        🕷️ Spider Stadia Store
      </button>
    )}
    <SkuList>
      {skus.map(sku => (
        <Sku {...sku} />
      ))}
    </SkuList>
  </main>
);

Home.style = {
  display: "block",
  fontSize: "14px",
  maxWidth: "800px",
  margin: "16px",
  backgroundImage: "url(/illufinch-violetsky-edited@2x.png)",
  backgroundPosition: "top 16px right 16px",
  backgroundRepeat: "no-repeat",
  backgroundSize: "64px",
};

const SkuList: JSX.FC<{ children: JSX.Renderable }> = ({ children }) => (
  <section>{children}</section>
);

SkuList.style = {
  alignItems: "end",
  display: "grid",
  gridAutoFlow: "dense row",
  gridTemplateAreas: `
    "  Title        Links        Prices       "
    "  Description  Description  Description  "
  `,
  gridTemplateColumns: "auto 200px 100px",
};

const Sku: JSX.FC<models.Sku> = sku => (
  <section>
    <Title name={sku.name} />
    <Description body={sku.description} />
    <Links sku={sku} />
    <Prices sku={sku} />
  </section>
);

const Title: JSX.FC<{ name: string }> = ({ name }) => <h2>{name}</h2>;

Title.style = {
  display: "block",
  gridColumn: "Title",
};

const Description: JSX.FC<{ body: string }> = ({ body }) => <p>{body}</p>;

Description.style = {
  display: "block",
  gridColumn: "Description",
  marginBottom: "32px",
  marginTop: "4px",
  padding: "8px",
  paddingRight: "32px",
};

const Links: JSX.FC<{ sku: models.Sku }> = ({ sku: { type, app, sku } }) => (
  <div>
    {type === "game" ? (
      <span>
        <a href={`http://stadia.google.com/player/${app}`}>🎮 play</a> or{" "}
      </span>
    ) : null}
    <a href={`https://stadia.google.com/store/details/${app}/sku/${sku}`}>
      🛒 shop
    </a>
  </div>
);

Links.style = {
  display: "block",
  gridColumn: "Links",
};

const Prices: JSX.FC<{ sku: models.Sku }> = ({ sku: { prices } }) => (
  <div>{models.Prices.prototype.render.call(prices)}</div>
);

Prices.style = {
  display: "block",
  gridColumn: "Prices",
  fontFamily: "JetBrains Mono, monospace",
  textAlign: "right",
};
