import { render } from "./render.js";
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
      render(
        "button",
        { onclick: async () => (await import("./spider.js")).spider() },
        "\uD83D\uDD77\uFE0Fspider stadia"
      ),
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
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiaW5kZXguanMiLCJzb3VyY2VSb290IjoiLi8iLCJzb3VyY2VzIjpbImluZGV4LnRzeCJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLEVBQUUsTUFBTSxFQUFFLE1BQU0sYUFBYSxDQUFDO0FBRXJDLFFBQVEsQ0FBQyxLQUFLLEdBQUcsaUJBQWlCLENBQUM7QUFDbkMsUUFBUSxDQUFDLElBQUksQ0FBQyxXQUFXLENBQ3ZCLE1BQU0sQ0FBQyxNQUFNLENBQUMsUUFBUSxDQUFDLGFBQWEsQ0FBQyxNQUFNLENBQUMsRUFBRTtJQUM1QyxHQUFHLEVBQUUsTUFBTTtJQUNYLElBQUksRUFBRSxvQ0FBb0M7Q0FDM0MsQ0FBQyxDQUNILENBQUM7QUFFRixNQUFNLElBQUksR0FBRyxDQUFDLEVBQUUsS0FBSyxFQUFPLEVBQUUsRUFBRSxDQUFDLENBQy9CLGlCQUNFLEtBQUssRUFBRTs7Ozs7Ozs7O0dBU1I7SUFFQyxxQ0FBd0I7SUFFdkIsTUFBTSxDQUFDLE1BQU0sRUFBRSxPQUFPLEVBQUUsRUFBRSxJQUFJLENBQzdCLG1CQUFRLE9BQU8sRUFBRSxLQUFLLElBQUksRUFBRSxDQUFDLENBQUMsTUFBTSxNQUFNLENBQUMsYUFBYSxDQUFDLENBQUMsQ0FBQyxNQUFNLEVBQUUsc0NBRTFELENBQ1Y7SUFFRCx3QkFDRyxLQUFLLENBQUMsR0FBRyxDQUFDLENBQUMsSUFBSSxFQUFFLEVBQUUsQ0FBQyxDQUNuQixPQUFDLElBQUksb0JBQUssSUFBSSxFQUFJLENBQ25CLENBQUMsQ0FDTSxDQUNMLENBQ1IsQ0FBQztBQUVGLE1BQU0sSUFBSSxHQUFHLENBQUMsRUFBRSxJQUFJLEVBQUUsV0FBVyxFQUFFLElBQUksRUFBRSxHQUFHLEVBQUUsR0FBRyxFQUFFLE1BQU0sRUFBRSxPQUFPLEVBQUUsRUFBRSxFQUFFLENBQUMsQ0FDdkU7SUFDRTtRQUNHLElBQUk7O1FBQ0osSUFBSSxLQUFLLE1BQU0sQ0FBQyxDQUFDLENBQUMsQ0FDakI7WUFDRSxjQUFHLElBQUksRUFBRSxtQ0FBbUMsR0FBRyxFQUFFLGtCQUFpQjtrQkFDN0QsQ0FDUixDQUFDLENBQUMsQ0FBQyxDQUNGLElBQUksQ0FDTDtRQUFFLEdBQUc7UUFDTixjQUFHLElBQUksRUFBRSwyQ0FBMkMsR0FBRyxRQUFRLEdBQUcsRUFBRSxvQkFFaEU7WUFFRDtJQUNMLGtCQUFJLFdBQVcsQ0FBSztJQUNuQixNQUFNLENBQUMsR0FBRyxDQUFDLENBQUMsS0FBSyxFQUFFLEVBQUUsQ0FBQyxDQUNyQjtRQUNFO1lBQ0csS0FBSyxDQUFDLElBQUk7O1lBQ1gsY0FDRSxJQUFJLEVBQUUsMkNBQTJDLEdBQUcsUUFBUSxLQUFLLENBQUMsR0FBRyxFQUFFLG9CQUdyRTtnQkFFRDtRQUNMLGtCQUFJLFdBQVcsQ0FBSyxDQUNaLENBQ1gsQ0FBQztJQUNELE9BQU8sQ0FBQyxHQUFHLENBQUMsQ0FBQyxNQUFNLEVBQUUsRUFBRSxDQUFDLENBQ3ZCO1FBQ0U7WUFDRyxNQUFNLENBQUMsSUFBSTs7WUFDWixjQUNFLElBQUksRUFBRSwyQ0FBMkMsR0FBRyxRQUFRLE1BQU0sQ0FBQyxHQUFHLEVBQUUsb0JBR3RFO2dCQUVEO1FBQ0wsa0JBQUksV0FBVyxDQUFLLENBQ1osQ0FDWCxDQUFDLENBQ00sQ0FDWCxDQUFDO0FBRUYsQ0FBQyxLQUFLLElBQUksRUFBRTtJQUNWLE1BQU0sSUFBSSxHQUFHLE1BQU0sQ0FBQyxNQUFNLENBQUMsTUFBTSxDQUFDLE1BQU0sS0FBSyxDQUFDLFlBQVksQ0FBQyxDQUFDLENBQUMsSUFBSSxFQUFFLENBQUMsQ0FBQztJQUNyRSxNQUFNLFFBQVEsR0FBRyxNQUFNLENBQUMsV0FBVyxDQUFDLElBQUksQ0FBQyxHQUFHLENBQUMsQ0FBQyxHQUFHLEVBQUUsRUFBRSxDQUFDLENBQUMsR0FBRyxDQUFDLEdBQUcsRUFBRSxHQUFHLENBQUMsQ0FBQyxDQUFDLENBQUM7SUFDdkUsTUFBTSxhQUFhLEdBQUcsSUFBSSxDQUFDLE1BQU0sQ0FBQyxDQUFDLEdBQUcsRUFBRSxFQUFFLENBQUMsR0FBRyxDQUFDLElBQUksS0FBSyxjQUFjLENBQUMsQ0FBQztJQUV4RSxNQUFNLEtBQUssR0FBRyxJQUFJO1NBQ2YsTUFBTSxDQUFDLENBQUMsR0FBRyxFQUFFLEVBQUUsQ0FBQyxHQUFHLENBQUMsSUFBSSxLQUFLLE1BQU0sQ0FBQztTQUNwQyxHQUFHLENBQUMsQ0FBQyxJQUFJLEVBQUUsRUFBRSxDQUNaLE1BQU0sQ0FBQyxNQUFNLENBQUMsSUFBSSxFQUFFO1FBQ2xCLElBQUksRUFBRSxJQUFJLENBQUMsTUFBTSxDQUFDLENBQUMsR0FBRyxFQUFFLEVBQUUsQ0FBQyxHQUFHLENBQUMsR0FBRyxLQUFLLElBQUksQ0FBQyxHQUFHLENBQUM7UUFDaEQsTUFBTSxFQUFFLElBQUksQ0FBQyxNQUFNLENBQ2pCLENBQUMsR0FBRyxFQUFFLEVBQUUsQ0FBQyxHQUFHLENBQUMsR0FBRyxLQUFLLElBQUksQ0FBQyxHQUFHLElBQUksR0FBRyxDQUFDLElBQUksS0FBSyxPQUFPLENBQ3REO1FBQ0QsT0FBTyxFQUFFLElBQUksQ0FBQyxNQUFNLENBQ2xCLENBQUMsR0FBRyxFQUFFLEVBQUUsQ0FBQyxHQUFHLENBQUMsR0FBRyxLQUFLLElBQUksQ0FBQyxHQUFHLElBQUksR0FBRyxDQUFDLElBQUksS0FBSyxRQUFRLENBQ3ZEO0tBQ0YsQ0FBQyxDQUNILENBQUM7SUFFSixRQUFRLENBQUMsSUFBSSxDQUFDLFdBQVcsQ0FBQyxPQUFDLElBQUksSUFBQyxLQUFLLEVBQUUsS0FBSyxHQUFJLENBQUMsQ0FBQztBQUNwRCxDQUFDLENBQUMsRUFBRSxDQUFDIiwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0IHsgcmVuZGVyIH0gZnJvbSBcIi4vcmVuZGVyLmpzXCI7XG5cbmRvY3VtZW50LnRpdGxlID0gXCJzdGFkaWEub2JzZXJ2ZXJcIjtcbmRvY3VtZW50LmhlYWQuYXBwZW5kQ2hpbGQoXG4gIE9iamVjdC5hc3NpZ24oZG9jdW1lbnQuY3JlYXRlRWxlbWVudChcImxpbmtcIiksIHtcbiAgICByZWw6IFwiaWNvblwiLFxuICAgIGhyZWY6IFwiL2lsbHVmaW5jaC12aW9sZXRza3ktZWRpdGVkQDJ4LnBuZ1wiLFxuICB9KVxuKTtcblxuY29uc3QgSG9tZSA9ICh7IGdhbWVzIH06IGFueSkgPT4gKFxuICA8bWFpblxuICAgIHN0eWxlPXtgXG4gICAgZm9udC1zaXplOiAxNHB4O1xuICAgIG1heC13aWR0aDogODAwcHg7XG4gICAgbWFyZ2luOiA4cHggMTZweDtcblxuICAgIGJhY2tncm91bmQtaW1hZ2U6IHVybCgvaWxsdWZpbmNoLXZpb2xldHNreS1lZGl0ZWRAMngucG5nKTtcbiAgICBiYWNrZ3JvdW5kLXBvc2l0aW9uOiB0b3AgMTZweCByaWdodCAxNnB4O1xuICAgIGJhY2tncm91bmQtcmVwZWF0OiBuby1yZXBlYXQ7XG4gICAgYmFja2dyb3VuZC1zaXplOiA2NHB4O1xuICBgfVxuICA+XG4gICAgPGgxPnN0YWRpYS5vYnNlcnZlcjwvaDE+XG5cbiAgICB7d2luZG93LmNocm9tZT8ucnVudGltZT8uaWQgJiYgKFxuICAgICAgPGJ1dHRvbiBvbmNsaWNrPXthc3luYyAoKSA9PiAoYXdhaXQgaW1wb3J0KFwiLi9zcGlkZXIuanNcIikpLnNwaWRlcigpfT5cbiAgICAgICAg8J+Vt++4j3NwaWRlciBzdGFkaWFcbiAgICAgIDwvYnV0dG9uPlxuICAgICl9XG5cbiAgICA8c2VjdGlvbj5cbiAgICAgIHtnYW1lcy5tYXAoKGdhbWUpID0+IChcbiAgICAgICAgPEdhbWUgey4uLmdhbWV9IC8+XG4gICAgICApKX1cbiAgICA8L3NlY3Rpb24+XG4gIDwvbWFpbj5cbik7XG5cbmNvbnN0IEdhbWUgPSAoeyBuYW1lLCBkZXNjcmlwdGlvbiwgdHlwZSwgc2t1LCBhcHAsIGFkZG9ucywgYnVuZGxlcyB9KSA9PiAoXG4gIDxzZWN0aW9uPlxuICAgIDxoMj5cbiAgICAgIHtuYW1lfSBbXG4gICAgICB7dHlwZSA9PT0gXCJnYW1lXCIgPyAoXG4gICAgICAgIDxzcGFuPlxuICAgICAgICAgIDxhIGhyZWY9e2BodHRwOi8vc3RhZGlhLmdvb2dsZS5jb20vcGxheWVyLyR7YXBwfWB9PmxhdW5jaCBnYW1lPC9hPiBvclxuICAgICAgICA8L3NwYW4+XG4gICAgICApIDogKFxuICAgICAgICB0eXBlXG4gICAgICApfXtcIiBcIn1cbiAgICAgIDxhIGhyZWY9e2BodHRwczovL3N0YWRpYS5nb29nbGUuY29tL3N0b3JlL2RldGFpbHMvJHthcHB9L3NrdS8ke3NrdX1gfT5cbiAgICAgICAgdmlldyBpbiBzdG9yZVxuICAgICAgPC9hPlxuICAgICAgXVxuICAgIDwvaDI+XG4gICAgPHA+e2Rlc2NyaXB0aW9ufTwvcD5cbiAgICB7YWRkb25zLm1hcCgoYWRkb24pID0+IChcbiAgICAgIDxzZWN0aW9uPlxuICAgICAgICA8aDM+XG4gICAgICAgICAge2FkZG9uLm5hbWV9IFtcbiAgICAgICAgICA8YVxuICAgICAgICAgICAgaHJlZj17YGh0dHBzOi8vc3RhZGlhLmdvb2dsZS5jb20vc3RvcmUvZGV0YWlscy8ke2FwcH0vc2t1LyR7YWRkb24uc2t1fWB9XG4gICAgICAgICAgPlxuICAgICAgICAgICAgdmlldyBpbiBzdG9yZVxuICAgICAgICAgIDwvYT5cbiAgICAgICAgICBdXG4gICAgICAgIDwvaDM+XG4gICAgICAgIDxwPntkZXNjcmlwdGlvbn08L3A+XG4gICAgICA8L3NlY3Rpb24+XG4gICAgKSl9XG4gICAge2J1bmRsZXMubWFwKChidW5kbGUpID0+IChcbiAgICAgIDxzZWN0aW9uPlxuICAgICAgICA8aDM+XG4gICAgICAgICAge2J1bmRsZS5uYW1lfSBbXG4gICAgICAgICAgPGFcbiAgICAgICAgICAgIGhyZWY9e2BodHRwczovL3N0YWRpYS5nb29nbGUuY29tL3N0b3JlL2RldGFpbHMvJHthcHB9L3NrdS8ke2J1bmRsZS5za3V9YH1cbiAgICAgICAgICA+XG4gICAgICAgICAgICB2aWV3IGluIHN0b3JlXG4gICAgICAgICAgPC9hPlxuICAgICAgICAgIF1cbiAgICAgICAgPC9oMz5cbiAgICAgICAgPHA+e2Rlc2NyaXB0aW9ufTwvcD5cbiAgICAgIDwvc2VjdGlvbj5cbiAgICApKX1cbiAgPC9zZWN0aW9uPlxuKTtcblxuKGFzeW5jICgpID0+IHtcbiAgY29uc3Qgc2t1cyA9IE9iamVjdC52YWx1ZXMoYXdhaXQgKGF3YWl0IGZldGNoKFwiL3NrdXMuanNvblwiKSkuanNvbigpKTtcbiAgY29uc3Qgc2t1c0J5SWQgPSBPYmplY3QuZnJvbUVudHJpZXMoc2t1cy5tYXAoKHNrdSkgPT4gW3NrdS5za3UsIHNrdV0pKTtcbiAgY29uc3Qgc3Vic2NyaXB0aW9ucyA9IHNrdXMuZmlsdGVyKChza3UpID0+IHNrdS50eXBlID09PSBcInN1YnNjcmlwdGlvblwiKTtcblxuICBjb25zdCBnYW1lcyA9IHNrdXNcbiAgICAuZmlsdGVyKChza3UpID0+IHNrdS50eXBlID09PSBcImdhbWVcIilcbiAgICAubWFwKChnYW1lKSA9PlxuICAgICAgT2JqZWN0LmFzc2lnbihnYW1lLCB7XG4gICAgICAgIHNrdXM6IHNrdXMuZmlsdGVyKChza3UpID0+IHNrdS5hcHAgPT09IGdhbWUuYXBwKSxcbiAgICAgICAgYWRkb25zOiBza3VzLmZpbHRlcihcbiAgICAgICAgICAoc2t1KSA9PiBza3UuYXBwID09PSBnYW1lLmFwcCAmJiBza3UudHlwZSA9PT0gXCJhZGRvblwiXG4gICAgICAgICksXG4gICAgICAgIGJ1bmRsZXM6IHNrdXMuZmlsdGVyKFxuICAgICAgICAgIChza3UpID0+IHNrdS5hcHAgPT09IGdhbWUuYXBwICYmIHNrdS50eXBlID09PSBcImJ1bmRsZVwiXG4gICAgICAgICksXG4gICAgICB9KVxuICAgICk7XG5cbiAgZG9jdW1lbnQuYm9keS5hcHBlbmRDaGlsZCg8SG9tZSBnYW1lcz17Z2FtZXN9IC8+KTtcbn0pKCk7XG4iXX0=
