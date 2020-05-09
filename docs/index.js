import { Prices } from "./index/data/models.js";
import { render } from "./index/render.js";
import { spider } from "./index/spider.js";
(async () => {
  // TODO: only do this in dev please
  let originalBuildTimestamp = await (await fetch("/timestamp.json")).json();
  setInterval(async () => {
    let currentTimestamp = await (await fetch("/timestamp.json")).json();
    if (currentTimestamp !== originalBuildTimestamp) {
      document.location.reload();
    }
  }, 1000);
})();
(async () => {
  document.title = "stadia.observer";
  document.head.appendChild(
    Object.assign(document.createElement("link"), {
      rel: "icon",
      href: "/illufinch-violetsky-edited@2x.png",
    })
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
  document.body.appendChild(render(Home, { games: games }));
})();
const Home = ({ games }) =>
  render(
    "main",
    {
      style: `
    font-size: 14px;
    max-width: 800px;
    margin: 8px 16px;

    background-image: url(/illufinch-violetsky-edited@2x.png);
    background-position: top 16px right 16px;
    background-repeat: no-repeat;
    background-size: 64px;
  `,
    },
    render("h1", null, "stadia.observer"),
    render("p", null, "does this work?"),
    render("hr", null),
    window.chrome?.runtime?.id &&
      render("button", { onclick: spider }, "\uD83D\uDD77\uFE0Fspider stadia"),
    render(
      "section",
      null,
      games.map((game) => render(Game, Object.assign({}, game)))
    )
  );
const Game = ({ name, description, type, sku, app, addons, bundles, prices }) =>
  render(
    "section",
    null,
    render(
      "h2",
      null,
      name,
      " [",
      type === "game"
        ? render(
            "span",
            null,
            render(
              "a",
              { href: `http://stadia.google.com/player/${app}` },
              "launch game"
            ),
            " or"
          )
        : type,
      " ",
      render(
        "a",
        { href: `https://stadia.google.com/store/details/${app}/sku/${sku}` },
        "view in store"
      ),
      " ",
      Prices.prototype.render.call(prices),
      "]"
    ),
    render("p", null, description),
    addons.map((addon) =>
      render(
        "section",
        null,
        render(
          "h3",
          null,
          render("span", { style: "font-weight: normal;" }, "\u2795"),
          " ",
          addon.name,
          " [",
          render(
            "a",
            {
              href: `https://stadia.google.com/store/details/${app}/sku/${addon.sku}`,
            },
            "view in store"
          ),
          " ",
          Prices.prototype.render.call(addon.prices),
          "]"
        ),
        render("p", null, description)
      )
    ),
    bundles.map((bundle) =>
      render(
        "section",
        null,
        render(
          "h3",
          null,
          render("span", { style: "font-weight: normal;" }, "\uD83D\uDCE6"),
          " ",
          bundle.name,
          " [",
          render(
            "a",
            {
              href: `https://stadia.google.com/store/details/${app}/sku/${bundle.sku}`,
            },
            "view in store"
          ),
          " ",
          Prices.prototype.render.call(bundle.prices),
          "]"
        ),
        render("p", null, description)
      )
    )
  );
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiaW5kZXguanMiLCJzb3VyY2VSb290IjoiLi8iLCJzb3VyY2VzIjpbImluZGV4LnRzeCJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLEVBQUUsTUFBTSxFQUFFLE1BQU0sd0JBQXdCLENBQUM7QUFDaEQsT0FBTyxFQUFFLE1BQU0sRUFBRSxNQUFNLG1CQUFtQixDQUFDO0FBQzNDLE9BQU8sRUFBRSxNQUFNLEVBQUUsTUFBTSxtQkFBbUIsQ0FBQztBQUUzQyxDQUFDLEtBQUssSUFBSSxFQUFFO0lBQ1YsbUNBQW1DO0lBQ25DLElBQUksc0JBQXNCLEdBQUcsTUFBTSxDQUFDLE1BQU0sS0FBSyxDQUFDLGlCQUFpQixDQUFDLENBQUMsQ0FBQyxJQUFJLEVBQUUsQ0FBQztJQUMzRSxXQUFXLENBQUMsS0FBSyxJQUFJLEVBQUU7UUFDckIsSUFBSSxnQkFBZ0IsR0FBRyxNQUFNLENBQUMsTUFBTSxLQUFLLENBQUMsaUJBQWlCLENBQUMsQ0FBQyxDQUFDLElBQUksRUFBRSxDQUFDO1FBQ3JFLElBQUksZ0JBQWdCLEtBQUssc0JBQXNCLEVBQUU7WUFDL0MsUUFBUSxDQUFDLFFBQVEsQ0FBQyxNQUFNLEVBQUUsQ0FBQztTQUM1QjtJQUNILENBQUMsRUFBRSxJQUFJLENBQUMsQ0FBQztBQUNYLENBQUMsQ0FBQyxFQUFFLENBQUM7QUFFTCxDQUFDLEtBQUssSUFBSSxFQUFFO0lBQ1YsUUFBUSxDQUFDLEtBQUssR0FBRyxpQkFBaUIsQ0FBQztJQUNuQyxRQUFRLENBQUMsSUFBSSxDQUFDLFdBQVcsQ0FDdkIsTUFBTSxDQUFDLE1BQU0sQ0FBQyxRQUFRLENBQUMsYUFBYSxDQUFDLE1BQU0sQ0FBQyxFQUFFO1FBQzVDLEdBQUcsRUFBRSxNQUFNO1FBQ1gsSUFBSSxFQUFFLG9DQUFvQztLQUMzQyxDQUFDLENBQ0gsQ0FBQztJQUVGLE1BQU0sSUFBSSxHQUFHLE1BQU0sQ0FBQyxNQUFNLENBQUMsTUFBTSxDQUFDLE1BQU0sS0FBSyxDQUFDLFlBQVksQ0FBQyxDQUFDLENB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RCxjQUNFLElBQUksRUFBRSwyQ0FBMkMsR0FBRyxRQUFRLEtBQUssQ0FBQyxHQUFHLEVBQUUsb0JBR3JFO1lBQUMsR0FBRztZQUNQLE1BQU0sQ0FBQyxTQUFTLENBQUMsTUFBTSxDQUFDLElBQUksQ0FBQyxLQUFLLENBQUMsTUFBTSxDQUFDO2dCQUN4QztRQUNMLGtCQUFJLFdBQVcsQ0FBSyxDQUNaLENBQ1gsQ0FBQztJQUNELE9BQU8sQ0FBQyxHQUFHLENBQUMsQ0FBQyxNQUFNLEVBQUUsRUFBRSxDQUFDLENBQ3ZCO1FBQ0U7WUFDRSxpQkFBTSxLQUFLLEVBQUMsc0JBQXNCLG1CQUFVOztZQUFFLE1BQU0sQ0FBQyxJQUFJOztZQUN6RCxjQUNFLElBQUksRUFBRSwyQ0FBMkMsR0FBRyxRQUFRLE1BQU0sQ0FBQyxHQUFHLEVBQUUsb0JBR3RFO1lBQUMsR0FBRztZQUNQLE1BQU0sQ0FBQyxTQUFTLENBQUMsTUFBTSxDQUFDLElBQUksQ0FBQyxNQUFNLENBQUMsTUFBTSxDQUFDO2dCQUN6QztRQUNMLGtCQUFJLFdBQVcsQ0FBSyxDQUNaLENBQ1gsQ0FBQyxDQUNNLENBQ1gsQ0FBQyIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCB7IFByaWNlcyB9IGZyb20gXCIuL2luZGV4L2RhdGEvbW9kZWxzLmpzXCI7XG5pbXBvcnQgeyByZW5kZXIgfSBmcm9tIFwiLi9pbmRleC9yZW5kZXIuanNcIjtcbmltcG9ydCB7IHNwaWRlciB9IGZyb20gXCIuL2luZGV4L3NwaWRlci5qc1wiO1xuXG4oYXN5bmMgKCkgPT4ge1xuICAvLyBUT0RPOiBvbmx5IGRvIHRoaXMgaW4gZGV2IHBsZWFzZVxuICBsZXQgb3JpZ2luYWxCdWlsZFRpbWVzdGFtcCA9IGF3YWl0IChhd2FpdCBmZXRjaChcIi90aW1lc3RhbXAuanNvblwiKSkuanNvbigpO1xuICBzZXRJbnRlcnZhbChhc3luYyAoKSA9PiB7XG4gICAgbGV0IGN1cnJlbnRUaW1lc3RhbXAgPSBhd2FpdCAoYXdhaXQgZmV0Y2goXCIvdGltZXN0YW1wLmpzb25cIikpLmpzb24oKTtcbiAgICBpZiAoY3VycmVudFRpbWVzdGFtcCAhPT0gb3JpZ2luYWxCdWlsZFRpbWVzdGFtcCkge1xuICAgICAgZG9jdW1lbnQubG9jYXRpb24ucmVsb2FkKCk7XG4gICAgfVxuICB9LCAxMDAwKTtcbn0pKCk7XG5cbihhc3luYyAoKSA9PiB7XG4gIGRvY3VtZW50LnRpdGxlID0gXCJzdGFkaWEub2JzZXJ2ZXJcIjtcbiAgZG9jdW1lbnQuaGVhZC5hcHBlbmRDaGlsZChcbiAgICBPYmplY3QuYXNzaWduKGRvY3VtZW50LmNyZWF0ZUVsZW1lbnQoXCJsaW5rXCIpLCB7XG4gICAgICByZWw6IFwiaWNvblwiLFxuICAgICAgaHJlZjogXCIvaWxsdWZpbmNoLXZpb2xldHNreS1lZGl0ZWRAMngucG5nXCIsXG4gICAgfSlcbiAgKTtcblxuICBjb25zdCBza3VzID0gT2JqZWN0LnZhbHVlcyhhd2FpdCAoYXdhaXQgZmV0Y2goXCIvc2t1cy5qc29uXCIpKS5qc29uKCkpO1xuICBjb25zdCBza3VzQnlJZCA9IE9iamVjdC5mcm9tRW50cmllcyhza3VzLm1hcCgoc2t1KSA9PiBbc2t1LnNrdSwgc2t1XSkpO1xuICBjb25zdCBzdWJzY3JpcHRpb25zID0gc2t1cy5maWx0ZXIoKHNrdSkgPT4gc2t1LnR5cGUgPT09IFwic3Vic2NyaXB0aW9uXCIpO1xuXG4gIGNvbnN0IGdhbWVzID0gc2t1c1xuICAgIC5maWx0ZXIoKHNrdSkgPT4gc2t1LnR5cGUgPT09IFwiZ2FtZVwiKVxuICAgIC5tYXAoKGdhbWUpID0+XG4gICAgICBPYmplY3QuYXNzaWduKGdhbWUsIHtcbiAgICAgICAgc2t1czogc2t1cy5maWx0ZXIoKHNrdSkgPT4gc2t1LmFwcCA9PT0gZ2FtZS5hcHApLFxuICAgICAgICBhZGRvbnM6IHNrdXMuZmlsdGVyKFxuICAgICAgICAgIChza3UpID0+IHNrdS5hcHAgPT09IGdhbWUuYXBwICYmIHNrdS50eXBlID09PSBcImFkZG9uXCJcbiAgICAgICAgKSxcbiAgICAgICAgYnVuZGxlczogc2t1cy5maWx0ZXIoXG4gICAgICAgICAgKHNrdSkgPT4gc2t1LmFwcCA9PT0gZ2FtZS5hcHAgJiYgc2t1LnR5cGUgPT09IFwiYnVuZGxlXCJcbiAgICAgICAgKSxcbiAgICAgIH0pXG4gICAgKTtcblxuICBkb2N1bWVudC5ib2R5LmFwcGVuZENoaWxkKDxIb21lIGdhbWVzPXtnYW1lc30gLz4pO1xufSkoKTtcblxuY29uc3QgSG9tZSA9ICh7IGdhbWVzIH06IGFueSkgPT4gKFxuICA8bWFpblxuICAgIHN0eWxlPXtgXG4gICAgZm9udC1zaXplOiAxNHB4O1xuICAgIG1heC13aWR0aDogODAwcHg7XG4gICAgbWFyZ2luOiA4cHggMTZweDtcblxuICAgIGJhY2tncm91bmQtaW1hZ2U6IHVybCgvaWxsdWZpbmNoLXZpb2xldHNreS1lZGl0ZWRAMngucG5nKTtcbiAgICBiYWNrZ3JvdW5kLXBvc2l0aW9uOiB0b3AgMTZweCByaWdodCAxNnB4O1xuICAgIGJhY2tncm91bmQtcmVwZWF0OiBuby1yZXBlYXQ7XG4gICAgYmFja2dyb3VuZC1zaXplOiA2NHB4O1xuICBgfVxuICA+XG4gICAgPGgxPnN0YWRpYS5vYnNlcnZlcjwvaDE+XG5cbiAgICA8cD5kb2VzIHRoaXMgd29yaz88L3A+XG5cbiAgICA8aHIgLz5cblxuICAgIHt3aW5kb3cuY2hyb21lPy5ydW50aW1lPy5pZCAmJiAoXG4gICAgICA8YnV0dG9uIG9uY2xpY2s9e3NwaWRlcn0+8J+Vt++4j3NwaWRlciBzdGFkaWE8L2J1dHRvbj5cbiAgICApfVxuXG4gICAgPHNlY3Rpb24+XG4gICAgICB7Z2FtZXMubWFwKChnYW1lKSA9PiAoXG4gICAgICAgIDxHYW1lIHsuLi5nYW1lfSAvPlxuICAgICAgKSl9XG4gICAgPC9zZWN0aW9uPlxuICA8L21haW4+XG4pO1xuXG5jb25zdCBHYW1lID0gKHtcbiAgbmFtZSxcbiAgZGVzY3JpcHRpb24sXG4gIHR5cGUsXG4gIHNrdSxcbiAgYXBwLFxuICBhZGRvbnMsXG4gIGJ1bmRsZXMsXG4gIHByaWNlcyxcbn0pID0+IChcbiAgPHNlY3Rpb24+XG4gICAgPGgyPlxuICAgICAge25hbWV9IFtcbiAgICAgIHt0eXBlID09PSBcImdhbWVcIiA/IChcbiAgICAgICAgPHNwYW4+XG4gICAgICAgICAgPGEgaHJlZj17YGh0dHA6Ly9zdGFkaWEuZ29vZ2xlLmNvbS9wbGF5ZXIvJHthcHB9YH0+bGF1bmNoIGdhbWU8L2E+IG9yXG4gICAgICAgIDwvc3Bhbj5cbiAgICAgICkgOiAoXG4gICAgICAgIHR5cGVcbiAgICAgICl9e1wiIFwifVxuICAgICAgPGEgaHJlZj17YGh0dHBzOi8vc3RhZGlhLmdvb2dsZS5jb20vc3RvcmUvZGV0YWlscy8ke2FwcH0vc2t1LyR7c2t1fWB9PlxuICAgICAgICB2aWV3IGluIHN0b3JlXG4gICAgICA8L2E+e1wiIFwifVxuICAgICAge1ByaWNlcy5wcm90b3R5cGUucmVuZGVyLmNhbGwocHJpY2VzKX1dXG4gICAgPC9oMj5cbiAgICA8cD57ZGVzY3JpcHRpb259PC9wPlxuICAgIHthZGRvbnMubWFwKChhZGRvbikgPT4gKFxuICAgICAgPHNlY3Rpb24+XG4gICAgICAgIDxoMz5cbiAgICAgICAgICA8c3BhbiBzdHlsZT1cImZvbnQtd2VpZ2h0OiBub3JtYWw7XCI+4p6VPC9zcGFuPiB7YWRkb24ubmFtZX0gW1xuICAgICAgICAgIDxhXG4gICAgICAgICAgICBocmVmPXtgaHR0cHM6Ly9zdGFkaWEuZ29vZ2xlLmNvbS9zdG9yZS9kZXRhaWxzLyR7YXBwfS9za3UvJHthZGRvbi5za3V9YH1cbiAgICAgICAgICA+XG4gICAgICAgICAgICB2aWV3IGluIHN0b3JlXG4gICAgICAgICAgPC9hPntcIiBcIn1cbiAgICAgICAgICB7UHJpY2VzLnByb3RvdHlwZS5yZW5kZXIuY2FsbChhZGRvbi5wcmljZXMpfV1cbiAgICAgICAgPC9oMz5cbiAgICAgICAgPHA+e2Rlc2NyaXB0aW9ufTwvcD5cbiAgICAgIDwvc2VjdGlvbj5cbiAgICApKX1cbiAgICB7YnVuZGxlcy5tYXAoKGJ1bmRsZSkgPT4gKFxuICAgICAgPHNlY3Rpb24+XG4gICAgICAgIDxoMz5cbiAgICAgICAgICA8c3BhbiBzdHlsZT1cImZvbnQtd2VpZ2h0OiBub3JtYWw7XCI+8J+Tpjwvc3Bhbj4ge2J1bmRsZS5uYW1lfSBbXG4gICAgICAgICAgPGFcbiAgICAgICAgICAgIGhyZWY9e2BodHRwczovL3N0YWRpYS5nb29nbGUuY29tL3N0b3JlL2RldGFpbHMvJHthcHB9L3NrdS8ke2J1bmRsZS5za3V9YH1cbiAgICAgICAgICA+XG4gICAgICAgICAgICB2aWV3IGluIHN0b3JlXG4gICAgICAgICAgPC9hPntcIiBcIn1cbiAgICAgICAgICB7UHJpY2VzLnByb3RvdHlwZS5yZW5kZXIuY2FsbChidW5kbGUucHJpY2VzKX1dXG4gICAgICAgIDwvaDM+XG4gICAgICAgIDxwPntkZXNjcmlwdGlvbn08L3A+XG4gICAgICA8L3NlY3Rpb24+XG4gICAgKSl9XG4gIDwvc2VjdGlvbj5cbik7XG4iXX0=
