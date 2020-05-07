import { render } from "./index.js";
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
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoicGFnZS5qcyIsInNvdXJjZVJvb3QiOiIuLyIsInNvdXJjZXMiOlsicGFnZS50c3giXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQUEsT0FBTyxFQUFFLE1BQU0sRUFBRSxNQUFNLFlBQVksQ0FBQztBQUVwQyxRQUFRLENBQUMsS0FBSyxHQUFHLGlCQUFpQixDQUFDO0FBQ25DLFFBQVEsQ0FBQyxJQUFJLENBQUMsV0FBVyxDQUN2QixNQUFNLENBQUMsTUFBTSxDQUFDLFFBQVEsQ0FBQyxhQUFhLENBQUMsTUFBTSxDQUFDLEVBQUU7SUFDNUMsR0FBRyxFQUFFLE1BQU07SUFDWCxJQUFJLEVBQUUsb0NBQW9DO0NBQzNDLENBQUMsQ0FDSCxDQUFDO0FBRUYsTUFBTSxJQUFJLEdBQUcsQ0FBQyxFQUFFLEtBQUssRUFBTyxFQUFFLEVBQUUsQ0FBQyxDQUMvQixpQkFDRSxLQUFLLEVBQUU7Ozs7Ozs7OztHQVNSO0lBRUMscUNBQXdCO0lBRXZCLE1BQU0sQ0FBQyxNQUFNLEVBQUUsT0FBTyxFQUFFLEVBQUUsSUFBSSxDQUM3QixtQkFBUSxPQUFPLEVBQUUsS0FBSyxJQUFJLEVBQUUsQ0FBQyxDQUFDLE1BQU0sTUFBTSxDQUFDLGFBQWEsQ0FBQyxDQUFDLENBQUMsTUFBTSxFQUFFLHNDQUUxRCxDQUNWO0lBRUQsd0JBQ0csS0FBSyxDQUFDLEdBQUcsQ0FBQyxDQUFDLElBQUksRUFBRSxFQUFFLENBQUMsQ0FDbkIsT0FBQyxJQUFJLG9CQUFLLElBQUksRUFBSSxDQUNuQixDQUFDLENBQ00sQ0FDTCxDQUNSLENBQUM7QUFFRixNQUFNLElBQUksR0FBRyxDQUFDLEVBQUUsSUFBSSxFQUFFLFdBQVcsRUFBRSxJQUFJLEVBQUUsR0FBRyxFQUFFLEdBQUcsRUFBRSxNQUFNLEVBQUUsT0FBTyxFQUFFLEVBQUUsRUFBRSxDQUFDLENBQ3ZFO0lBQ0U7UUFDRyxJQUFJOztRQUNKLElBQUksS0FBSyxNQUFNLENBQUMsQ0FBQyxDQUFDLENBQ2pCO1lBQ0UsY0FBRyxJQUFJLEVBQUUsbUNBQW1DLEdBQUcsRUFBRSxrQkFBaUI7a0JBQzdELENBQ1IsQ0FBQyxDQUFDLENBQUMsQ0FDRixJQUFJLENBQ0w7UUFBRSxHQUFHO1FBQ04sY0FBRyxJQUFJLEVBQUUsMkNBQTJDLEdBQUcsUUFBUSxHQUFHLEVBQUUsb0JBRWhFO1lBRUQ7SUFDTCxrQkFBSSxXQUFXLENBQUs7SUFDbkIsTUFBTSxDQUFDLEdBQUcsQ0FBQyxDQUFDLEtBQUssRUFBRSxFQUFFLENBQUMsQ0FDckI7UUFDRTtZQUNFLGlCQUFNLEtBQUssRUFBQyxzQkFBc0IsYUFBUzs7WUFBRSxLQUFLLENBQUMsSUFBSTs7WUFDdkQsY0FDRSxJQUFJLEVBQUUsMkNBQTJDLEdBQUcsUUFBUSxLQUFLLENBQUMsR0FBRyxFQUFFLG9CQUdyRTtnQkFFRDtRQUNMLGtCQUFJLFdBQVcsQ0FBSyxDQUNaLENBQ1gsQ0FBQztJQUNELE9BQU8sQ0FBQyxHQUFHLENBQUMsQ0FBQyxNQUFNLEVBQUUsRUFBRSxDQUFDLENBQ3ZCO1FBQ0U7WUFDRSxpQkFBTSxLQUFLLEVBQUMsc0JBQXNCLG1CQUFVOztZQUFFLE1BQU0sQ0FBQyxJQUFJOztZQUN6RCxjQUNFLElBQUksRUFBRSwyQ0FBMkMsR0FBRyxRQUFRLE1BQU0sQ0FBQyxHQUFHLEVBQUUsb0JBR3RFO2dCQUVEO1FBQ0wsa0JBQUksV0FBVyxDQUFLLENBQ1osQ0FDWCxDQUFDLENBQ00sQ0FDWCxDQUFDO0FBRUYsQ0FBQyxLQUFLLElBQUksRUFBRTtJQUNWLE1BQU0sSUFBSSxHQUFHLE1BQU0sQ0FBQyxNQUFNLENBQUMsTUFBTSxDQUFDLE1BQU0sS0FBSyxDQUFDLFlBQVksQ0FBQyxDQUFDLENBQUMsSUFBSSxFQUFFLENBQUMsQ0FBQztJQUNyRSxNQUFNLFFBQVEsR0FBRyxNQUFNLENBQUMsV0FBVyxDQUFDLElBQUksQ0FBQyxHQUFHLENBQUMsQ0FBQyxHQUFHLEVBQUUsRUFBRSxDQUFDLENBQUMsR0FBRyxDQUFDLEdBQUcsRUFBRSxHQUFHLENBQUMsQ0FBQyxDQUFDLENBQUM7SUFDdkUsTUFBTSxhQUFhLEdBQUcsSUFBSSxDQUFDLE1BQU0sQ0FBQyxDQUFDLEdBQUcsRUFBRSxFQUFFLENBQUMsR0FBRyxDQUFDLElBQUksS0FBSyxjQUFjLENBQUMsQ0FBQztJQUV4RSxNQUFNLEtBQUssR0FBRyxJQUFJO1NBQ2YsTUFBTSxDQUFDLENBQUMsR0FBRyxFQUFFLEVBQUUsQ0FBQyxHQUFHLENBQUMsSUFBSSxLQUFLLE1BQU0sQ0FBQztTQUNwQyxHQUFHLENBQUMsQ0FBQyxJQUFJLEVBQUUsRUFBRSxDQUNaLE1BQU0sQ0FBQyxNQUFNLENBQUMsSUFBSSxFQUFFO1FBQ2xCLElBQUksRUFBRSxJQUFJLENBQUMsTUFBTSxDQUFDLENBQUMsR0FBRyxFQUFFLEVBQUUsQ0FBQyxHQUFHLENBQUMsR0FBRyxLQUFLLElBQUksQ0FBQyxHQUFHLENBQUM7UUFDaEQsTUFBTSxFQUFFLElBQUksQ0FBQyxNQUFNLENBQ2pCLENBQUMsR0FBRyxFQUFFLEVBQUUsQ0FBQyxHQUFHLENBQUMsR0FBRyxLQUFLLElBQUksQ0FBQyxHQUFHLElBQUksR0FBRyxDQUFDLElBQUksS0FBSyxPQUFPLENBQ3REO1FBQ0QsT0FBTyxFQUFFLElBQUksQ0FBQyxNQUFNLENBQ2xCLENBQUMsR0FBRyxFQUFFLEVBQUUsQ0FBQyxHQUFHLENBQUMsR0FBRyxLQUFLLElBQUksQ0FBQyxHQUFHLElBQUksR0FBRyxDQUFDLElBQUksS0FBSyxRQUFRLENBQ3ZEO0tBQ0YsQ0FBQyxDQUNILENBQUM7SUFFSixRQUFRLENBQUMsSUFBSSxDQUFDLFdBQVcsQ0FBQyxPQUFDLElBQUksSUFBQyxLQUFLLEVBQUUsS0FBSyxHQUFJLENBQUMsQ0FBQztBQUNwRCxDQUFDLENBQUMsRUFBRSxDQUFDIiwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0IHsgcmVuZGVyIH0gZnJvbSBcIi4vaW5kZXguanNcIjtcblxuZG9jdW1lbnQudGl0bGUgPSBcInN0YWRpYS5vYnNlcnZlclwiO1xuZG9jdW1lbnQuaGVhZC5hcHBlbmRDaGlsZChcbiAgT2JqZWN0LmFzc2lnbihkb2N1bWVudC5jcmVhdGVFbGVtZW50KFwibGlua1wiKSwge1xuICAgIHJlbDogXCJpY29uXCIsXG4gICAgaHJlZjogXCIvaWxsdWZpbmNoLXZpb2xldHNreS1lZGl0ZWRAMngucG5nXCIsXG4gIH0pXG4pO1xuXG5jb25zdCBIb21lID0gKHsgZ2FtZXMgfTogYW55KSA9PiAoXG4gIDxtYWluXG4gICAgc3R5bGU9e2BcbiAgICBmb250LXNpemU6IDE0cHg7XG4gICAgbWF4LXdpZHRoOiA4MDBweDtcbiAgICBtYXJnaW46IDhweCAxNnB4O1xuXG4gICAgYmFja2dyb3VuZC1pbWFnZTogdXJsKC9pbGx1ZmluY2gtdmlvbGV0c2t5LWVkaXRlZEAyeC5wbmcpO1xuICAgIGJhY2tncm91bmQtcG9zaXRpb246IHRvcCAxNnB4IHJpZ2h0IDE2cHg7XG4gICAgYmFja2dyb3VuZC1yZXBlYXQ6IG5vLXJlcGVhdDtcbiAgICBiYWNrZ3JvdW5kLXNpemU6IDY0cHg7XG4gIGB9XG4gID5cbiAgICA8aDE+c3RhZGlhLm9ic2VydmVyPC9oMT5cblxuICAgIHt3aW5kb3cuY2hyb21lPy5ydW50aW1lPy5pZCAmJiAoXG4gICAgICA8YnV0dG9uIG9uY2xpY2s9e2FzeW5jICgpID0+IChhd2FpdCBpbXBvcnQoXCIuL3NwaWRlci5qc1wiKSkuc3BpZGVyKCl9PlxuICAgICAgICDwn5W377iPc3BpZGVyIHN0YWRpYVxuICAgICAgPC9idXR0b24+XG4gICAgKX1cblxuICAgIDxzZWN0aW9uPlxuICAgICAge2dhbWVzLm1hcCgoZ2FtZSkgPT4gKFxuICAgICAgICA8R2FtZSB7Li4uZ2FtZX0gLz5cbiAgICAgICkpfVxuICAgIDwvc2VjdGlvbj5cbiAgPC9tYWluPlxuKTtcblxuY29uc3QgR2FtZSA9ICh7IG5hbWUsIGRlc2NyaXB0aW9uLCB0eXBlLCBza3UsIGFwcCwgYWRkb25zLCBidW5kbGVzIH0pID0+IChcbiAgPHNlY3Rpb24+XG4gICAgPGgyPlxuICAgICAge25hbWV9IFtcbiAgICAgIHt0eXBlID09PSBcImdhbWVcIiA/IChcbiAgICAgICAgPHNwYW4+XG4gICAgICAgICAgPGEgaHJlZj17YGh0dHA6Ly9zdGFkaWEuZ29vZ2xlLmNvbS9wbGF5ZXIvJHthcHB9YH0+bGF1bmNoIGdhbWU8L2E+IG9yXG4gICAgICAgIDwvc3Bhbj5cbiAgICAgICkgOiAoXG4gICAgICAgIHR5cGVcbiAgICAgICl9e1wiIFwifVxuICAgICAgPGEgaHJlZj17YGh0dHBzOi8vc3RhZGlhLmdvb2dsZS5jb20vc3RvcmUvZGV0YWlscy8ke2FwcH0vc2t1LyR7c2t1fWB9PlxuICAgICAgICB2aWV3IGluIHN0b3JlXG4gICAgICA8L2E+XG4gICAgICBdXG4gICAgPC9oMj5cbiAgICA8cD57ZGVzY3JpcHRpb259PC9wPlxuICAgIHthZGRvbnMubWFwKChhZGRvbikgPT4gKFxuICAgICAgPHNlY3Rpb24+XG4gICAgICAgIDxoMz5cbiAgICAgICAgICA8c3BhbiBzdHlsZT1cImZvbnQtd2VpZ2h0OiBub3JtYWw7XCI+4p6VPC9zcGFuPiB7YWRkb24ubmFtZX0gW1xuICAgICAgICAgIDxhXG4gICAgICAgICAgICBocmVmPXtgaHR0cHM6Ly9zdGFkaWEuZ29vZ2xlLmNvbS9zdG9yZS9kZXRhaWxzLyR7YXBwfS9za3UvJHthZGRvbi5za3V9YH1cbiAgICAgICAgICA+XG4gICAgICAgICAgICB2aWV3IGluIHN0b3JlXG4gICAgICAgICAgPC9hPlxuICAgICAgICAgIF1cbiAgICAgICAgPC9oMz5cbiAgICAgICAgPHA+e2Rlc2NyaXB0aW9ufTwvcD5cbiAgICAgIDwvc2VjdGlvbj5cbiAgICApKX1cbiAgICB7YnVuZGxlcy5tYXAoKGJ1bmRsZSkgPT4gKFxuICAgICAgPHNlY3Rpb24+XG4gICAgICAgIDxoMz5cbiAgICAgICAgICA8c3BhbiBzdHlsZT1cImZvbnQtd2VpZ2h0OiBub3JtYWw7XCI+8J+Tpjwvc3Bhbj4ge2J1bmRsZS5uYW1lfSBbXG4gICAgICAgICAgPGFcbiAgICAgICAgICAgIGhyZWY9e2BodHRwczovL3N0YWRpYS5nb29nbGUuY29tL3N0b3JlL2RldGFpbHMvJHthcHB9L3NrdS8ke2J1bmRsZS5za3V9YH1cbiAgICAgICAgICA+XG4gICAgICAgICAgICB2aWV3IGluIHN0b3JlXG4gICAgICAgICAgPC9hPlxuICAgICAgICAgIF1cbiAgICAgICAgPC9oMz5cbiAgICAgICAgPHA+e2Rlc2NyaXB0aW9ufTwvcD5cbiAgICAgIDwvc2VjdGlvbj5cbiAgICApKX1cbiAgPC9zZWN0aW9uPlxuKTtcblxuKGFzeW5jICgpID0+IHtcbiAgY29uc3Qgc2t1cyA9IE9iamVjdC52YWx1ZXMoYXdhaXQgKGF3YWl0IGZldGNoKFwiL3NrdXMuanNvblwiKSkuanNvbigpKTtcbiAgY29uc3Qgc2t1c0J5SWQgPSBPYmplY3QuZnJvbUVudHJpZXMoc2t1cy5tYXAoKHNrdSkgPT4gW3NrdS5za3UsIHNrdV0pKTtcbiAgY29uc3Qgc3Vic2NyaXB0aW9ucyA9IHNrdXMuZmlsdGVyKChza3UpID0+IHNrdS50eXBlID09PSBcInN1YnNjcmlwdGlvblwiKTtcblxuICBjb25zdCBnYW1lcyA9IHNrdXNcbiAgICAuZmlsdGVyKChza3UpID0+IHNrdS50eXBlID09PSBcImdhbWVcIilcbiAgICAubWFwKChnYW1lKSA9PlxuICAgICAgT2JqZWN0LmFzc2lnbihnYW1lLCB7XG4gICAgICAgIHNrdXM6IHNrdXMuZmlsdGVyKChza3UpID0+IHNrdS5hcHAgPT09IGdhbWUuYXBwKSxcbiAgICAgICAgYWRkb25zOiBza3VzLmZpbHRlcihcbiAgICAgICAgICAoc2t1KSA9PiBza3UuYXBwID09PSBnYW1lLmFwcCAmJiBza3UudHlwZSA9PT0gXCJhZGRvblwiXG4gICAgICAgICksXG4gICAgICAgIGJ1bmRsZXM6IHNrdXMuZmlsdGVyKFxuICAgICAgICAgIChza3UpID0+IHNrdS5hcHAgPT09IGdhbWUuYXBwICYmIHNrdS50eXBlID09PSBcImJ1bmRsZVwiXG4gICAgICAgICksXG4gICAgICB9KVxuICAgICk7XG5cbiAgZG9jdW1lbnQuYm9keS5hcHBlbmRDaGlsZCg8SG9tZSBnYW1lcz17Z2FtZXN9IC8+KTtcbn0pKCk7XG4iXX0=
