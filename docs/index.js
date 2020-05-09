import { render } from "./index/render.js";
import { spider } from "./index/spider.js";
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
    window.chrome?.runtime?.id &&
      render("button", { onclick: spider }, "\uD83D\uDD77\uFE0Fspider stadia"),
    render(
      "section",
      null,
      games.map((game) => render(Game, Object.assign({}, game)))
    )
  );
const Game = ({ name, description, type, sku, app, addons, bundles }) =>
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
          "]"
        ),
        render("p", null, description)
      )
    )
  );
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiaW5kZXguanMiLCJzb3VyY2VSb290IjoiLi8iLCJzb3VyY2VzIjpbImluZGV4LnRzeCJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLEVBQUUsTUFBTSxFQUFFLE1BQU0sbUJBQW1CLENBQUM7QUFDM0MsT0FBTyxFQUFFLE1BQU0sRUFBRSxNQUFNLG1CQUFtQixDQUFDO0FBRTNDLENBQUMsS0FBSyxJQUFJLEVBQUU7SUFDVixRQUFRLENBQUMsS0FBSyxHQUFHLGlCQUFpQixDQUFDO0lBQ25DLFFBQVEsQ0FBQyxJQUFJLENBQUMsV0FBVyxDQUN2QixNQUFNLENBQUMsTUFBTSxDQUFDLFFBQVEsQ0FBQyxhQUFhLENBQUMsTUFBTSxDQUFDLEVBQUU7UUFDNUMsR0FBRyxFQUFFLE1BQU07UUFDWCxJQUFJLEVBQUUsb0NBQW9DO0tBQzNDLENBQUMsQ0FDSCxDQUFDO0lBRUYsTUFBTSxJQUFJLEdBQUcsTUFBTSxDQUFDLE1BQU0sQ0FBQyxNQUFNLENBQUMsTUFBTSxLQUFLLENBQUMsWUFBWSxDQUFDLENBQUMsQ0FBQyxJQUFJLEVBQUUsQ0FBQyxDQUFDO0lBQ3JFLE1BQU0sUUFBUSxHQUFHLE1BQU0sQ0FBQyxXQUFXLENBQUMsSUFBSSxDQUFDLEdBQUcsQ0FBQyxDQUFDLEdBQUcsRUFBRSxFQUFFLENBQUMsQ0FBQyxHQUFHLENBQUMsR0FBRyxFQUFFLEdBQUcsQ0FBQyxDQUFDLENBQUMsQ0FBQztJQUN2RSxNQUFNLGFBQWEsR0FBRyxJQUFJLENBQUMsTUFBTSxDQUFDLENBQUMsR0FBRyxFQUFFLEVBQUUsQ0FBQyxHQUFHLENBQUMsSUFBSSxLQUFLLGNBQWMsQ0FBQyxDQUFDO0lBRXhFLE1BQU0sS0FBSyxHQUFHLElBQUk7U0FDZixNQUFNLENBQUMsQ0FBQyxHQUFHLEVBQUUsRUFBRSxDQUFDLEdBQUcsQ0FBQyxJQUFJLEtBQUssTUFBTSxDQUFDO1NBQ3BDLEdBQUcsQ0FBQyxDQUFDLElBQUksRUFBRSxFQUFFLENBQ1osTUFBTSxDQUFDLE1BQU0sQ0FBQyxJQUFJLEVBQUU7UUFDbEIsSUFBSSxFQUFFLElBQUksQ0FBQyxNQUFNLENBQUMsQ0FBQyxHQUFHLEVBQUUsRUFBRSxDQUFDLEdBQUcsQ0FBQyxHQUFHLEtBQUssSUFBSSxDQUFDLEdBQUcsQ0FBQztRQUNoRCxNQUFNLEVBQUUsSUFBSSxDQUFDLE1BQU0sQ0FDakIsQ0FBQyxHQUFHLEVBQUUsRUFBRSxDQUFDLEdBQUcsQ0FBQyxHQUFHLEtBQUssSUFBSSxDQUFDLEdBQUcsSUFBSSxHQUFHLENBQUMsSUFBSSxLQUFLLE9BQU8sQ0FDdEQ7UUFDRCxPQUFPLEVBQUUsSUFBSSxDQUFDLE1BQU0sQ0FDbEIsQ0FBQyxHQUFHLEVBQUUsRUFBRSxDQUFDLEdBQUcsQ0FBQyxHQUFHLEtBQUssSUFBSSxDQUFDLEdBQUcsSUFBSSxHQUFHLENBQUMsSUFBSSxLQUFLLFFBQVEsQ0FDdkQ7S0FDRixDQUFDLENBQ0gsQ0FBQztJQUVKLFFBQVEsQ0FBQyxJQUFJLENBQUMsV0FBVyxDQUFDLE9BQUMsSUFBSSxJQUFDLEtBQUssRUFBRSxLQUFLLEdBQUksQ0FBQyxDQUFDO0FBQ3BELENBQUMsQ0FBQyxFQUFFLENBQUM7QUFFTCxNQUFNLElBQUksR0FBRyxDQUFDLEVBQUUsS0FBSyxFQUFPLEVBQUUsRUFBRSxDQUFDLENBQy9CLGlCQUNFLEtBQUssRUFBRTs7Ozs7Ozs7O0dBU1I7SUFFQyxxQ0FBd0I7SUFFdkIsTUFBTSxDQUFDLE1BQU0sRUFBRSxPQUFPLEVBQUUsRUFBRSxJQUFJLENBQzdCLG1CQUFRLE9BQU8sRUFBRSxNQUFNLHNDQUEyQixDQUNuRDtJQUVELHdCQUNHLEtBQUssQ0FBQyxHQUFHLENBQUMsQ0FBQyxJQUFJLEVBQUUsRUFBRSxDQUFDLENBQ25CLE9BQUMsSUFBSSxvQkFBSyxJQUFJLEVBQUksQ0FDbkIsQ0FBQyxDQUNNLENBQ0wsQ0FDUixDQUFDO0FBRUYsTUFBTSxJQUFJLEdBQUcsQ0FBQyxFQUFFLElBQUksRUFBRSxXQUFXLEVBQUUsSUFBSSxFQUFFLEdBQUcsRUFBRSxHQUFHLEVBQUUsTUFBTSxFQUFFLE9BQU8sRUFBRSxFQUFFLEVBQUUsQ0FBQyxDQUN2RTtJQUNFO1FBQ0csSUFBSTs7UUFDSixJQUFJLEtBQUssTUFBTSxDQUFDLENBQUMsQ0FBQyxDQUNqQjtZQUNFLGNBQUcsSUFBSSxFQUFFLG1DQUFtQyxHQUFHLEVBQUUsa0JBQWlCO2tCQUM3RCxDQUNSLENBQUMsQ0FBQyxDQUFDLENBQ0YsSUFBSSxDQUNMO1FBQUUsR0FBRztRQUNOLGNBQUcsSUFBSSxFQUFFLDJDQUEyQyxHQUFHLFFBQVEsR0FBRyxFQUFFLG9CQUVoRTtZQUVEO0lBQ0wsa0JBQUksV0FBVyxDQUFLO0lBQ25CLE1BQU0sQ0FBQyxHQUFHLENBQUMsQ0FBQyxLQUFLLEVBQUUsRUFBRSxDQUFDLENBQ3JCO1FBQ0U7WUFDRSxpQkFBTSxLQUFLLEVBQUMsc0JBQXNCLGFBQVM7O1lBQUUsS0FBSyxDQUFDLElBQUk7O1lBQ3ZELGNBQ0UsSUFBSSxFQUFFLDJDQUEyQyxHQUFHLFFBQVEsS0FBSyxDQUFDLEdBQUcsRUFBRSxvQkFHckU7Z0JBRUQ7UUFDTCxrQkFBSSxXQUFXLENBQUssQ0FDWixDQUNYLENBQUM7SUFDRCxPQUFPLENBQUMsR0FBRyxDQUFDLENBQUMsTUFBTSxFQUFFLEVBQUUsQ0FBQyxDQUN2QjtRQUNFO1lBQ0UsaUJBQU0sS0FBSyxFQUFDLHNCQUFzQixtQkFBVTs7WUFBRSxNQUFNLENBQUMsSUFBSTs7WUFDekQsY0FDRSxJQUFJLEVBQUUsMkNBQTJDLEdBQUcsUUFBUSxNQUFNLENBQUMsR0FBRyxFQUFFLG9CQUd0RTtnQkFFRDtRQUNMLGtCQUFJLFdBQVcsQ0FBSyxDQUNaLENBQ1gsQ0FBQyxDQUNNLENBQ1gsQ0FBQyIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCB7IHJlbmRlciB9IGZyb20gXCIuL2luZGV4L3JlbmRlci5qc1wiO1xuaW1wb3J0IHsgc3BpZGVyIH0gZnJvbSBcIi4vaW5kZXgvc3BpZGVyLmpzXCI7XG5cbihhc3luYyAoKSA9PiB7XG4gIGRvY3VtZW50LnRpdGxlID0gXCJzdGFkaWEub2JzZXJ2ZXJcIjtcbiAgZG9jdW1lbnQuaGVhZC5hcHBlbmRDaGlsZChcbiAgICBPYmplY3QuYXNzaWduKGRvY3VtZW50LmNyZWF0ZUVsZW1lbnQoXCJsaW5rXCIpLCB7XG4gICAgICByZWw6IFwiaWNvblwiLFxuICAgICAgaHJlZjogXCIvaWxsdWZpbmNoLXZpb2xldHNreS1lZGl0ZWRAMngucG5nXCIsXG4gICAgfSlcbiAgKTtcblxuICBjb25zdCBza3VzID0gT2JqZWN0LnZhbHVlcyhhd2FpdCAoYXdhaXQgZmV0Y2goXCIvc2t1cy5qc29uXCIpKS5qc29uKCkpO1xuICBjb25zdCBza3VzQnlJZCA9IE9iamVjdC5mcm9tRW50cmllcyhza3VzLm1hcCgoc2t1KSA9PiBbc2t1LnNrdSwgc2t1XSkpO1xuICBjb25zdCBzdWJzY3JpcHRpb25zID0gc2t1cy5maWx0ZXIoKHNrdSkgPT4gc2t1LnR5cGUgPT09IFwic3Vic2NyaXB0aW9uXCIpO1xuXG4gIGNvbnN0IGdhbWVzID0gc2t1c1xuICAgIC5maWx0ZXIoKHNrdSkgPT4gc2t1LnR5cGUgPT09IFwiZ2FtZVwiKVxuICAgIC5tYXAoKGdhbWUpID0+XG4gICAgICBPYmplY3QuYXNzaWduKGdhbWUsIHtcbiAgICAgICAgc2t1czogc2t1cy5maWx0ZXIoKHNrdSkgPT4gc2t1LmFwcCA9PT0gZ2FtZS5hcHApLFxuICAgICAgICBhZGRvbnM6IHNrdXMuZmlsdGVyKFxuICAgICAgICAgIChza3UpID0+IHNrdS5hcHAgPT09IGdhbWUuYXBwICYmIHNrdS50eXBlID09PSBcImFkZG9uXCJcbiAgICAgICAgKSxcbiAgICAgICAgYnVuZGxlczogc2t1cy5maWx0ZXIoXG4gICAgICAgICAgKHNrdSkgPT4gc2t1LmFwcCA9PT0gZ2FtZS5hcHAgJiYgc2t1LnR5cGUgPT09IFwiYnVuZGxlXCJcbiAgICAgICAgKSxcbiAgICAgIH0pXG4gICAgKTtcblxuICBkb2N1bWVudC5ib2R5LmFwcGVuZENoaWxkKDxIb21lIGdhbWVzPXtnYW1lc30gLz4pO1xufSkoKTtcblxuY29uc3QgSG9tZSA9ICh7IGdhbWVzIH06IGFueSkgPT4gKFxuICA8bWFpblxuICAgIHN0eWxlPXtgXG4gICAgZm9udC1zaXplOiAxNHB4O1xuICAgIG1heC13aWR0aDogODAwcHg7XG4gICAgbWFyZ2luOiA4cHggMTZweDtcblxuICAgIGJhY2tncm91bmQtaW1hZ2U6IHVybCgvaWxsdWZpbmNoLXZpb2xldHNreS1lZGl0ZWRAMngucG5nKTtcbiAgICBiYWNrZ3JvdW5kLXBvc2l0aW9uOiB0b3AgMTZweCByaWdodCAxNnB4O1xuICAgIGJhY2tncm91bmQtcmVwZWF0OiBuby1yZXBlYXQ7XG4gICAgYmFja2dyb3VuZC1zaXplOiA2NHB4O1xuICBgfVxuICA+XG4gICAgPGgxPnN0YWRpYS5vYnNlcnZlcjwvaDE+XG5cbiAgICB7d2luZG93LmNocm9tZT8ucnVudGltZT8uaWQgJiYgKFxuICAgICAgPGJ1dHRvbiBvbmNsaWNrPXtzcGlkZXJ9PvCflbfvuI9zcGlkZXIgc3RhZGlhPC9idXR0b24+XG4gICAgKX1cblxuICAgIDxzZWN0aW9uPlxuICAgICAge2dhbWVzLm1hcCgoZ2FtZSkgPT4gKFxuICAgICAgICA8R2FtZSB7Li4uZ2FtZX0gLz5cbiAgICAgICkpfVxuICAgIDwvc2VjdGlvbj5cbiAgPC9tYWluPlxuKTtcblxuY29uc3QgR2FtZSA9ICh7IG5hbWUsIGRlc2NyaXB0aW9uLCB0eXBlLCBza3UsIGFwcCwgYWRkb25zLCBidW5kbGVzIH0pID0+IChcbiAgPHNlY3Rpb24+XG4gICAgPGgyPlxuICAgICAge25hbWV9IFtcbiAgICAgIHt0eXBlID09PSBcImdhbWVcIiA/IChcbiAgICAgICAgPHNwYW4+XG4gICAgICAgICAgPGEgaHJlZj17YGh0dHA6Ly9zdGFkaWEuZ29vZ2xlLmNvbS9wbGF5ZXIvJHthcHB9YH0+bGF1bmNoIGdhbWU8L2E+IG9yXG4gICAgICAgIDwvc3Bhbj5cbiAgICAgICkgOiAoXG4gICAgICAgIHR5cGVcbiAgICAgICl9e1wiIFwifVxuICAgICAgPGEgaHJlZj17YGh0dHBzOi8vc3RhZGlhLmdvb2dsZS5jb20vc3RvcmUvZGV0YWlscy8ke2FwcH0vc2t1LyR7c2t1fWB9PlxuICAgICAgICB2aWV3IGluIHN0b3JlXG4gICAgICA8L2E+XG4gICAgICBdXG4gICAgPC9oMj5cbiAgICA8cD57ZGVzY3JpcHRpb259PC9wPlxuICAgIHthZGRvbnMubWFwKChhZGRvbikgPT4gKFxuICAgICAgPHNlY3Rpb24+XG4gICAgICAgIDxoMz5cbiAgICAgICAgICA8c3BhbiBzdHlsZT1cImZvbnQtd2VpZ2h0OiBub3JtYWw7XCI+4p6VPC9zcGFuPiB7YWRkb24ubmFtZX0gW1xuICAgICAgICAgIDxhXG4gICAgICAgICAgICBocmVmPXtgaHR0cHM6Ly9zdGFkaWEuZ29vZ2xlLmNvbS9zdG9yZS9kZXRhaWxzLyR7YXBwfS9za3UvJHthZGRvbi5za3V9YH1cbiAgICAgICAgICA+XG4gICAgICAgICAgICB2aWV3IGluIHN0b3JlXG4gICAgICAgICAgPC9hPlxuICAgICAgICAgIF1cbiAgICAgICAgPC9oMz5cbiAgICAgICAgPHA+e2Rlc2NyaXB0aW9ufTwvcD5cbiAgICAgIDwvc2VjdGlvbj5cbiAgICApKX1cbiAgICB7YnVuZGxlcy5tYXAoKGJ1bmRsZSkgPT4gKFxuICAgICAgPHNlY3Rpb24+XG4gICAgICAgIDxoMz5cbiAgICAgICAgICA8c3BhbiBzdHlsZT1cImZvbnQtd2VpZ2h0OiBub3JtYWw7XCI+8J+Tpjwvc3Bhbj4ge2J1bmRsZS5uYW1lfSBbXG4gICAgICAgICAgPGFcbiAgICAgICAgICAgIGhyZWY9e2BodHRwczovL3N0YWRpYS5nb29nbGUuY29tL3N0b3JlL2RldGFpbHMvJHthcHB9L3NrdS8ke2J1bmRsZS5za3V9YH1cbiAgICAgICAgICA+XG4gICAgICAgICAgICB2aWV3IGluIHN0b3JlXG4gICAgICAgICAgPC9hPlxuICAgICAgICAgIF1cbiAgICAgICAgPC9oMz5cbiAgICAgICAgPHA+e2Rlc2NyaXB0aW9ufTwvcD5cbiAgICAgIDwvc2VjdGlvbj5cbiAgICApKX1cbiAgPC9zZWN0aW9uPlxuKTtcbiJdfQ==
