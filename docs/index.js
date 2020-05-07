import { render } from "./index/render.js";
import { spider } from "./index/spider.js";
document.title = "stadia.observer";
document.head.appendChild(
  Object.assign(document.createElement("link"), {
    rel: "icon",
    href: "/illufinch-violetsky-edited@2x.png",
  })
);
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
  document.body.appendChild(render(Home, { games: games }));
})();
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiaW5kZXguanMiLCJzb3VyY2VSb290IjoiLi8iLCJzb3VyY2VzIjpbImluZGV4LnRzeCJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLEVBQUUsTUFBTSxFQUFFLE1BQU0sbUJBQW1CLENBQUM7QUFDM0MsT0FBTyxFQUFFLE1BQU0sRUFBRSxNQUFNLG1CQUFtQixDQUFDO0FBRTNDLFFBQVEsQ0FBQyxLQUFLLEdBQUcsaUJBQWlCLENBQUM7QUFDbkMsUUFBUSxDQUFDLElBQUksQ0FBQyxXQUFXLENBQ3ZCLE1BQU0sQ0FBQyxNQUFNLENBQUMsUUFBUSxDQUFDLGFBQWEsQ0FBQyxNQUFNLENBQUMsRUFBRTtJQUM1QyxHQUFHLEVBQUUsTUFBTTtJQUNYLElBQUksRUFBRSxvQ0FBb0M7Q0FDM0MsQ0FBQyxDQUNILENBQUM7QUFFRixNQUFNLElBQUksR0FBRyxDQUFDLEVBQUUsS0FBSyxFQUFPLEVBQUUsRUFBRSxDQUFDLENBQy9CLGlCQUNFLEtBQUssRUFBRTs7Ozs7Ozs7O0dBU1I7SUFFQyxxQ0FBd0I7SUFFdkIsTUFBTSxDQUFDLE1BQU0sRUFBRSxPQUFPLEVBQUUsRUFBRSxJQUFJLENBQzdCLG1CQUFRLE9BQU8sRUFBRSxNQUFNLHNDQUEyQixDQUNuRDtJQUVELHdCQUNHLEtBQUssQ0FBQyxHQUFHLENBQUMsQ0FBQyxJQUFJLEVBQUUsRUFBRSxDQUFDLENBQ25CLE9BQUMsSUFBSSxvQkFBSyxJQUFJLEVBQUksQ0FDbkIsQ0FBQyxDQUNNLENBQ0wsQ0FDUixDQUFDO0FBRUYsTUFBTSxJQUFJLEdBQUcsQ0FBQyxFQUFFLElBQUksRUFBRSxXQUFXLEVBQUUsSUFBSSxFQUFFLEdBQUcsRUFBRSxHQUFHLEVBQUUsTUFBTSxFQUFFLE9BQU8sRUFBRSxFQUFFLEVBQUUsQ0FBQyxDQUN2RTtJQUNFO1FBQ0csSUFBSTs7UUFDSixJQUFJLEtBQUssTUFBTSxDQUFDLENBQUMsQ0FBQyxDQUNqQjtZQUNFLGNBQUcsSUFBSSxFQUFFLG1DQUFtQyxHQUFHLEVBQUUsa0JBQWlCO2tCQUM3RCxDQUNSLENBQUMsQ0FBQyxDQUFDLENBQ0YsSUFBSSxDQUNMO1FBQUUsR0FBRztRQUNOLGNBQUcsSUFBSSxFQUFFLDJDQUEyQyxHQUFHLFFBQVEsR0FBRyxFQUFFLG9CQUVoRTtZQUVEO0lBQ0wsa0JBQUksV0FBVyxDQUFLO0lBQ25CLE1BQU0sQ0FBQyxHQUFHLENBQUMsQ0FBQyxLQUFLLEVBQUUsRUFBRSxDQUFDLENBQ3JCO1FBQ0U7WUFDRSxpQkFBTSxLQUFLLEVBQUMsc0JBQXNCLGFBQVM7O1lBQUUsS0FBSyxDQUFDLElBQUk7O1lBQ3ZELGNBQ0UsSUFBSSxFQUFFLDJDQUEyQyxHQUFHLFFBQVEsS0FBSyxDQUFDLEdBQUcsRUFBRSxvQkFHckU7Z0JBRUQ7UUFDTCxrQkFBSSxXQUFXLENBQUssQ0FDWixDQUNYLENBQUM7SUFDRCxPQUFPLENBQUMsR0FBRyxDQUFDLENBQUMsTUFBTSxFQUFFLEVBQUUsQ0FBQyxDQUN2QjtRQUNFO1lBQ0UsaUJBQU0sS0FBSyxFQUFDLHNCQUFzQixtQkFBVTs7WUFBRSxNQUFNLENBQUMsSUFBSTs7WUFDekQsY0FDRSxJQUFJLEVBQUUsMkNBQTJDLEdBQUcsUUFBUSxNQUFNLENBQUMsR0FBRyxFQUFFLG9CQUd0RTtnQkFFRDtRQUNMLGtCQUFJLFdBQVcsQ0FBSyxDQUNaLENBQ1gsQ0FBQyxDQUNNLENBQ1gsQ0FBQztBQUVGLENBQUMsS0FBSyxJQUFJLEVBQUU7SUFDVixNQUFNLElBQUksR0FBRyxNQUFNLENBQUMsTUFBTSxDQUFDLE1BQU0sQ0FBQyxNQUFNLEtBQUssQ0FBQyxZQUFZLENBQUMsQ0FBQyxDQUFDLElBQUksRUFBRSxDQUFDLENBQUM7SUFDckUsTUFBTSxRQUFRLEdBQUcsTUFBTSxDQUFDLFdBQVcsQ0FBQyxJQUFJLENBQUMsR0FBRyxDQUFDLENBQUMsR0FBRyxFQUFFLEVBQUUsQ0FBQyxDQUFDLEdBQUcsQ0FBQyxHQUFHLEVBQUUsR0FBRyxDQUFDLENBQUMsQ0FBQyxDQUFDO0lBQ3ZFLE1BQU0sYUFBYSxHQUFHLElBQUksQ0FBQyxNQUFNLENBQUMsQ0FBQyxHQUFHLEVBQUUsRUFBRSxDQUFDLEdBQUcsQ0FBQyxJQUFJLEtBQUssY0FBYyxDQUFDLENBQUM7SUFFeEUsTUFBTSxLQUFLLEdBQUcsSUFBSTtTQUNmLE1BQU0sQ0FBQyxDQUFDLEdBQUcsRUFBRSxFQUFFLENBQUMsR0FBRyxDQUFDLElBQUksS0FBSyxNQUFNLENBQUM7U0FDcEMsR0FBRyxDQUFDLENBQUMsSUFBSSxFQUFFLEVBQUUsQ0FDWixNQUFNLENBQUMsTUFBTSxDQUFDLElBQUksRUFBRTtRQUNsQixJQUFJLEVBQUUsSUFBSSxDQUFDLE1BQU0sQ0FBQyxDQUFDLEdBQUcsRUFBRSxFQUFFLENBQUMsR0FBRyxDQUFDLEdBQUcsS0FBSyxJQUFJLENBQUMsR0FBRyxDQUFDO1FBQ2hELE1BQU0sRUFBRSxJQUFJLENBQUMsTUFBTSxDQUNqQixDQUFDLEdBQUcsRUFBRSxFQUFFLENBQUMsR0FBRyxDQUFDLEdBQUcsS0FBSyxJQUFJLENBQUMsR0FBRyxJQUFJLEdBQUcsQ0FBQyxJQUFJLEtBQUssT0FBTyxDQUN0RDtRQUNELE9BQU8sRUFBRSxJQUFJLENBQUMsTUFBTSxDQUNsQixDQUFDLEdBQUcsRUFBRSxFQUFFLENBQUMsR0FBRyxDQUFDLEdBQUcsS0FBSyxJQUFJLENBQUMsR0FBRyxJQUFJLEdBQUcsQ0FBQyxJQUFJLEtBQUssUUFBUSxDQUN2RDtLQUNGLENBQUMsQ0FDSCxDQUFDO0lBRUosUUFBUSxDQUFDLElBQUksQ0FBQyxXQUFXLENBQUMsT0FBQyxJQUFJLElBQUMsS0FBSyxFQUFFLEtBQUssR0FBSSxDQUFDLENBQUM7QUFDcEQsQ0FBQyxDQUFDLEVBQUUsQ0FBQyIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCB7IHJlbmRlciB9IGZyb20gXCIuL2luZGV4L3JlbmRlci5qc1wiO1xuaW1wb3J0IHsgc3BpZGVyIH0gZnJvbSBcIi4vaW5kZXgvc3BpZGVyLmpzXCI7XG5cbmRvY3VtZW50LnRpdGxlID0gXCJzdGFkaWEub2JzZXJ2ZXJcIjtcbmRvY3VtZW50LmhlYWQuYXBwZW5kQ2hpbGQoXG4gIE9iamVjdC5hc3NpZ24oZG9jdW1lbnQuY3JlYXRlRWxlbWVudChcImxpbmtcIiksIHtcbiAgICByZWw6IFwiaWNvblwiLFxuICAgIGhyZWY6IFwiL2lsbHVmaW5jaC12aW9sZXRza3ktZWRpdGVkQDJ4LnBuZ1wiLFxuICB9KVxuKTtcblxuY29uc3QgSG9tZSA9ICh7IGdhbWVzIH06IGFueSkgPT4gKFxuICA8bWFpblxuICAgIHN0eWxlPXtgXG4gICAgZm9udC1zaXplOiAxNHB4O1xuICAgIG1heC13aWR0aDogODAwcHg7XG4gICAgbWFyZ2luOiA4cHggMTZweDtcblxuICAgIGJhY2tncm91bmQtaW1hZ2U6IHVybCgvaWxsdWZpbmNoLXZpb2xldHNreS1lZGl0ZWRAMngucG5nKTtcbiAgICBiYWNrZ3JvdW5kLXBvc2l0aW9uOiB0b3AgMTZweCByaWdodCAxNnB4O1xuICAgIGJhY2tncm91bmQtcmVwZWF0OiBuby1yZXBlYXQ7XG4gICAgYmFja2dyb3VuZC1zaXplOiA2NHB4O1xuICBgfVxuICA+XG4gICAgPGgxPnN0YWRpYS5vYnNlcnZlcjwvaDE+XG5cbiAgICB7d2luZG93LmNocm9tZT8ucnVudGltZT8uaWQgJiYgKFxuICAgICAgPGJ1dHRvbiBvbmNsaWNrPXtzcGlkZXJ9PvCflbfvuI9zcGlkZXIgc3RhZGlhPC9idXR0b24+XG4gICAgKX1cblxuICAgIDxzZWN0aW9uPlxuICAgICAge2dhbWVzLm1hcCgoZ2FtZSkgPT4gKFxuICAgICAgICA8R2FtZSB7Li4uZ2FtZX0gLz5cbiAgICAgICkpfVxuICAgIDwvc2VjdGlvbj5cbiAgPC9tYWluPlxuKTtcblxuY29uc3QgR2FtZSA9ICh7IG5hbWUsIGRlc2NyaXB0aW9uLCB0eXBlLCBza3UsIGFwcCwgYWRkb25zLCBidW5kbGVzIH0pID0+IChcbiAgPHNlY3Rpb24+XG4gICAgPGgyPlxuICAgICAge25hbWV9IFtcbiAgICAgIHt0eXBlID09PSBcImdhbWVcIiA/IChcbiAgICAgICAgPHNwYW4+XG4gICAgICAgICAgPGEgaHJlZj17YGh0dHA6Ly9zdGFkaWEuZ29vZ2xlLmNvbS9wbGF5ZXIvJHthcHB9YH0+bGF1bmNoIGdhbWU8L2E+IG9yXG4gICAgICAgIDwvc3Bhbj5cbiAgICAgICkgOiAoXG4gICAgICAgIHR5cGVcbiAgICAgICl9e1wiIFwifVxuICAgICAgPGEgaHJlZj17YGh0dHBzOi8vc3RhZGlhLmdvb2dsZS5jb20vc3RvcmUvZGV0YWlscy8ke2FwcH0vc2t1LyR7c2t1fWB9PlxuICAgICAgICB2aWV3IGluIHN0b3JlXG4gICAgICA8L2E+XG4gICAgICBdXG4gICAgPC9oMj5cbiAgICA8cD57ZGVzY3JpcHRpb259PC9wPlxuICAgIHthZGRvbnMubWFwKChhZGRvbikgPT4gKFxuICAgICAgPHNlY3Rpb24+XG4gICAgICAgIDxoMz5cbiAgICAgICAgICA8c3BhbiBzdHlsZT1cImZvbnQtd2VpZ2h0OiBub3JtYWw7XCI+4p6VPC9zcGFuPiB7YWRkb24ubmFtZX0gW1xuICAgICAgICAgIDxhXG4gICAgICAgICAgICBocmVmPXtgaHR0cHM6Ly9zdGFkaWEuZ29vZ2xlLmNvbS9zdG9yZS9kZXRhaWxzLyR7YXBwfS9za3UvJHthZGRvbi5za3V9YH1cbiAgICAgICAgICA+XG4gICAgICAgICAgICB2aWV3IGluIHN0b3JlXG4gICAgICAgICAgPC9hPlxuICAgICAgICAgIF1cbiAgICAgICAgPC9oMz5cbiAgICAgICAgPHA+e2Rlc2NyaXB0aW9ufTwvcD5cbiAgICAgIDwvc2VjdGlvbj5cbiAgICApKX1cbiAgICB7YnVuZGxlcy5tYXAoKGJ1bmRsZSkgPT4gKFxuICAgICAgPHNlY3Rpb24+XG4gICAgICAgIDxoMz5cbiAgICAgICAgICA8c3BhbiBzdHlsZT1cImZvbnQtd2VpZ2h0OiBub3JtYWw7XCI+8J+Tpjwvc3Bhbj4ge2J1bmRsZS5uYW1lfSBbXG4gICAgICAgICAgPGFcbiAgICAgICAgICAgIGhyZWY9e2BodHRwczovL3N0YWRpYS5nb29nbGUuY29tL3N0b3JlL2RldGFpbHMvJHthcHB9L3NrdS8ke2J1bmRsZS5za3V9YH1cbiAgICAgICAgICA+XG4gICAgICAgICAgICB2aWV3IGluIHN0b3JlXG4gICAgICAgICAgPC9hPlxuICAgICAgICAgIF1cbiAgICAgICAgPC9oMz5cbiAgICAgICAgPHA+e2Rlc2NyaXB0aW9ufTwvcD5cbiAgICAgIDwvc2VjdGlvbj5cbiAgICApKX1cbiAgPC9zZWN0aW9uPlxuKTtcblxuKGFzeW5jICgpID0+IHtcbiAgY29uc3Qgc2t1cyA9IE9iamVjdC52YWx1ZXMoYXdhaXQgKGF3YWl0IGZldGNoKFwiL3NrdXMuanNvblwiKSkuanNvbigpKTtcbiAgY29uc3Qgc2t1c0J5SWQgPSBPYmplY3QuZnJvbUVudHJpZXMoc2t1cy5tYXAoKHNrdSkgPT4gW3NrdS5za3UsIHNrdV0pKTtcbiAgY29uc3Qgc3Vic2NyaXB0aW9ucyA9IHNrdXMuZmlsdGVyKChza3UpID0+IHNrdS50eXBlID09PSBcInN1YnNjcmlwdGlvblwiKTtcblxuICBjb25zdCBnYW1lcyA9IHNrdXNcbiAgICAuZmlsdGVyKChza3UpID0+IHNrdS50eXBlID09PSBcImdhbWVcIilcbiAgICAubWFwKChnYW1lKSA9PlxuICAgICAgT2JqZWN0LmFzc2lnbihnYW1lLCB7XG4gICAgICAgIHNrdXM6IHNrdXMuZmlsdGVyKChza3UpID0+IHNrdS5hcHAgPT09IGdhbWUuYXBwKSxcbiAgICAgICAgYWRkb25zOiBza3VzLmZpbHRlcihcbiAgICAgICAgICAoc2t1KSA9PiBza3UuYXBwID09PSBnYW1lLmFwcCAmJiBza3UudHlwZSA9PT0gXCJhZGRvblwiXG4gICAgICAgICksXG4gICAgICAgIGJ1bmRsZXM6IHNrdXMuZmlsdGVyKFxuICAgICAgICAgIChza3UpID0+IHNrdS5hcHAgPT09IGdhbWUuYXBwICYmIHNrdS50eXBlID09PSBcImJ1bmRsZVwiXG4gICAgICAgICksXG4gICAgICB9KVxuICAgICk7XG5cbiAgZG9jdW1lbnQuYm9keS5hcHBlbmRDaGlsZCg8SG9tZSBnYW1lcz17Z2FtZXN9IC8+KTtcbn0pKCk7XG4iXX0=
