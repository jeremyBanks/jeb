import "./index/jsx.js";
import { Prices } from "./index/data/models.js";
import { spider } from "./index/spider.js";
(async () => {
  let originalBuildTimestamp = await (
    await fetch("/last-build-timestamp.json")
  ).json();
  setInterval(async () => {
    let currentTimestamp = await (
      await fetch("/last-build-timestamp.json")
    ).json();
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
  document.body.appendChild(JSX.createElement(Home, { games: games }));
})();
const Home = ({ games }) =>
  JSX.createElement(
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
    JSX.createElement("h1", null, "stadia.observer"),
    JSX.createElement("p", null, "does this work?"),
    JSX.createElement("hr", null),
    window.chrome?.runtime?.id &&
      JSX.createElement(
        "button",
        { onclick: spider },
        "\uD83D\uDD77\uFE0Fspider stadia"
      ),
    JSX.createElement(
      "section",
      null,
      games.map((game) => JSX.createElement(Game, Object.assign({}, game)))
    )
  );
const Game = ({ name, description, type, sku, app, addons, bundles, prices }) =>
  JSX.createElement(
    "section",
    null,
    JSX.createElement(
      "h2",
      null,
      name,
      " [",
      type === "game"
        ? JSX.createElement(
            "span",
            null,
            JSX.createElement(
              "a",
              { href: `http://stadia.google.com/player/${app}` },
              "launch game"
            ),
            " or"
          )
        : type,
      " ",
      JSX.createElement(
        "a",
        { href: `https://stadia.google.com/store/details/${app}/sku/${sku}` },
        "view in store"
      ),
      " ",
      Prices.prototype.render.call(prices),
      "]"
    ),
    JSX.createElement("p", null, description),
    addons.map((addon) =>
      JSX.createElement(
        "section",
        null,
        JSX.createElement(
          "h3",
          null,
          JSX.createElement(
            "span",
            { style: "font-weight: normal;" },
            "\u2795"
          ),
          " ",
          addon.name,
          " [",
          JSX.createElement(
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
        JSX.createElement("p", null, description)
      )
    ),
    bundles.map((bundle) =>
      JSX.createElement(
        "section",
        null,
        JSX.createElement(
          "h3",
          null,
          JSX.createElement(
            "span",
            { style: "font-weight: normal;" },
            "\uD83D\uDCE6"
          ),
          " ",
          bundle.name,
          " [",
          JSX.createElement(
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
        JSX.createElement("p", null, description)
      )
    )
  );
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiaW5kZXguanMiLCJzb3VyY2VSb290IjoiLi8iLCJzb3VyY2VzIjpbImluZGV4LnRzeCJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLGdCQUFnQixDQUFDO0FBRXhCLE9BQU8sRUFBRSxNQUFNLEVBQUUsTUFBTSx3QkFBd0IsQ0FBQztBQUNoRCxPQUFPLEVBQUUsTUFBTSxFQUFFLE1BQU0sbUJBQW1CLENBQUM7QUFFM0MsQ0FBQyxLQUFLLElBQUksRUFBRTtJQUNWLElBQUksc0JBQXNCLEdBQUcsTUFBTSxDQUNqQyxNQUFNLEtBQUssQ0FBQyw0QkFBNEIsQ0FBQyxDQUMxQyxDQUFDLElBQUksRUFBRSxDQUFDO0lBQ1QsV0FBVyxDQUFDLEtBQUssSUFBSSxFQUFFO1FBQ3JCLElBQUksZ0JBQWdCLEdBQUcsTUFBTSxDQUMzQixNQUFNLEtBQUssQ0FBQyw0QkFBNEIsQ0FBQyxDQUMxQyxDQUFDLElBQUksRUFBRSxDQUFDO1FBQ1QsSUFBSSxnQkFBZ0IsS0FBSyxzQkFBc0IsRUFBRTtZQUMvQyxRQUFRLENBQUMsUUFBUSxDQUFDLE1BQU0sRUFBRSxDQUFDO1NBQzVCO0lBQ0gsQ0FBQyxFQUFFLElBQUksQ0FBQyxDQUFDO0FBQ1gsQ0FBQyxDQUFDLEVBQUUsQ0FBQztBQUVMLENBQUMsS0FBSyxJQUFJLEVBQUU7SUFDVixRQUFRLENBQUMsS0FBSyxHQUFHLGlCQUFpQixDQUFDO0lBQ25DLFFBQVEsQ0FBQyxJQUFJLENBQUMsV0FBVyxDQUN2QixNQUFNLENBQUMsTUFBTSxDQUFDLFFBQVEsQ0FBQyxhQUFhLENBQUMsTUFBTSxDQUFDLEVBQUU7UUFDNUMsR0FBRyxFQUFFLE1BQU07UUFDWCxJQUFJLEVBQUUsb0NBQW9DO0tBQzNDLENBQUMsQ0FDSCxDQUFDO0lBRUYsTUFBTSxJQUFJLEdBQUcsTUFBTSxDQUFDLE1BQU0sQ0FBQyxNQUFNLENBQUMsTUFBTSxLQUFLLENBQUMsWUFBWSxDQUFDLENBQUMsQ0FBQyxJQUFJLEVBQUUsQ0FBQyxDQUFDO0lBQ3JFLE1B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QUcsUUFBUSxLQUFLLENBQUMsR0FBRyxFQUFFLG9CQUdyRTtZQUFDLEdBQUc7WUFDUCxNQUFNLENBQUMsU0FBUyxDQUFDLE1BQU0sQ0FBQyxJQUFJLENBQUMsS0FBSyxDQUFDLE1BQU0sQ0FBQztnQkFDeEM7UUFDTCw2QkFBSSxXQUFXLENBQUssQ0FDWixDQUNYLENBQUM7SUFDRCxPQUFPLENBQUMsR0FBRyxDQUFDLENBQUMsTUFBTSxFQUFFLEVBQUUsQ0FBQyxDQUN2QjtRQUNFO1lBQ0UsNEJBQU0sS0FBSyxFQUFDLHNCQUFzQixtQkFBVTs7WUFBRSxNQUFNLENBQUMsSUFBSTs7WUFDekQseUJBQ0UsSUFBSSxFQUFFLDJDQUEyQyxHQUFHLFFBQVEsTUFBTSxDQUFDLEdBQUcsRUFBRSxvQkFHdEU7WUFBQyxHQUFHO1lBQ1AsTUFBTSxDQUFDLFNBQVMsQ0FBQyxNQUFNLENBQUMsSUFBSSxDQUFDLE1BQU0sQ0FBQyxNQUFNLENBQUM7Z0JBQ3pDO1FBQ0wsNkJBQUksV0FBVyxDQUFLLENBQ1osQ0FDWCxDQUFDLENBQ00sQ0FDWCxDQUFDIiwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0IFwiLi9pbmRleC9qc3guanNcIjtcblxuaW1wb3J0IHsgUHJpY2VzIH0gZnJvbSBcIi4vaW5kZXgvZGF0YS9tb2RlbHMuanNcIjtcbmltcG9ydCB7IHNwaWRlciB9IGZyb20gXCIuL2luZGV4L3NwaWRlci5qc1wiO1xuXG4oYXN5bmMgKCkgPT4ge1xuICBsZXQgb3JpZ2luYWxCdWlsZFRpbWVzdGFtcCA9IGF3YWl0IChcbiAgICBhd2FpdCBmZXRjaChcIi9sYXN0LWJ1aWxkLXRpbWVzdGFtcC5qc29uXCIpXG4gICkuanNvbigpO1xuICBzZXRJbnRlcnZhbChhc3luYyAoKSA9PiB7XG4gICAgbGV0IGN1cnJlbnRUaW1lc3RhbXAgPSBhd2FpdCAoXG4gICAgICBhd2FpdCBmZXRjaChcIi9sYXN0LWJ1aWxkLXRpbWVzdGFtcC5qc29uXCIpXG4gICAgKS5qc29uKCk7XG4gICAgaWYgKGN1cnJlbnRUaW1lc3RhbXAgIT09IG9yaWdpbmFsQnVpbGRUaW1lc3RhbXApIHtcbiAgICAgIGRvY3VtZW50LmxvY2F0aW9uLnJlbG9hZCgpO1xuICAgIH1cbiAgfSwgMTAwMCk7XG59KSgpO1xuXG4oYXN5bmMgKCkgPT4ge1xuICBkb2N1bWVudC50aXRsZSA9IFwic3RhZGlhLm9ic2VydmVyXCI7XG4gIGRvY3VtZW50LmhlYWQuYXBwZW5kQ2hpbGQoXG4gICAgT2JqZWN0LmFzc2lnbihkb2N1bWVudC5jcmVhdGVFbGVtZW50KFwibGlua1wiKSwge1xuICAgICAgcmVsOiBcImljb25cIixcbiAgICAgIGhyZWY6IFwiL2lsbHVmaW5jaC12aW9sZXRza3ktZWRpdGVkQDJ4LnBuZ1wiLFxuICAgIH0pXG4gICk7XG5cbiAgY29uc3Qgc2t1cyA9IE9iamVjdC52YWx1ZXMoYXdhaXQgKGF3YWl0IGZldGNoKFwiL3NrdXMuanNvblwiKSkuanNvbigpKTtcbiAgY29uc3Qgc2t1c0J5SWQgPSBPYmplY3QuZnJvbUVudHJpZXMoc2t1cy5tYXAoKHNrdSkgPT4gW3NrdS5za3UsIHNrdV0pKTtcbiAgY29uc3Qgc3Vic2NyaXB0aW9ucyA9IHNrdXMuZmlsdGVyKChza3UpID0+IHNrdS50eXBlID09PSBcInN1YnNjcmlwdGlvblwiKTtcblxuICBjb25zdCBnYW1lcyA9IHNrdXNcbiAgICAuZmlsdGVyKChza3UpID0+IHNrdS50eXBlID09PSBcImdhbWVcIilcbiAgICAubWFwKChnYW1lKSA9PlxuICAgICAgT2JqZWN0LmFzc2lnbihnYW1lLCB7XG4gICAgICAgIHNrdXM6IHNrdXMuZmlsdGVyKChza3UpID0+IHNrdS5hcHAgPT09IGdhbWUuYXBwKSxcbiAgICAgICAgYWRkb25zOiBza3VzLmZpbHRlcihcbiAgICAgICAgICAoc2t1KSA9PiBza3UuYXBwID09PSBnYW1lLmFwcCAmJiBza3UudHlwZSA9PT0gXCJhZGRvblwiXG4gICAgICAgICksXG4gICAgICAgIGJ1bmRsZXM6IHNrdXMuZmlsdGVyKFxuICAgICAgICAgIChza3UpID0+IHNrdS5hcHAgPT09IGdhbWUuYXBwICYmIHNrdS50eXBlID09PSBcImJ1bmRsZVwiXG4gICAgICAgICksXG4gICAgICB9KVxuICAgICk7XG5cbiAgZG9jdW1lbnQuYm9keS5hcHBlbmRDaGlsZCg8SG9tZSBnYW1lcz17Z2FtZXN9IC8+KTtcbn0pKCk7XG5cbmNvbnN0IEhvbWUgPSAoeyBnYW1lcyB9OiBhbnkpID0+IChcbiAgPG1haW5cbiAgICBzdHlsZT17YFxuICAgIGZvbnQtc2l6ZTogMTRweDtcbiAgICBtYXgtd2lkdGg6IDgwMHB4O1xuICAgIG1hcmdpbjogOHB4IDE2cHg7XG5cbiAgICBiYWNrZ3JvdW5kLWltYWdlOiB1cmwoL2lsbHVmaW5jaC12aW9sZXRza3ktZWRpdGVkQDJ4LnBuZyk7XG4gICAgYmFja2dyb3VuZC1wb3NpdGlvbjogdG9wIDE2cHggcmlnaHQgMTZweDtcbiAgICBiYWNrZ3JvdW5kLXJlcGVhdDogbm8tcmVwZWF0O1xuICAgIGJhY2tncm91bmQtc2l6ZTogNjRweDtcbiAgYH1cbiAgPlxuICAgIDxoMT5zdGFkaWEub2JzZXJ2ZXI8L2gxPlxuXG4gICAgPHA+ZG9lcyB0aGlzIHdvcms/PC9wPlxuXG4gICAgPGhyIC8+XG5cbiAgICB7d2luZG93LmNocm9tZT8ucnVudGltZT8uaWQgJiYgKFxuICAgICAgPGJ1dHRvbiBvbmNsaWNrPXtzcGlkZXJ9PvCflbfvuI9zcGlkZXIgc3RhZGlhPC9idXR0b24+XG4gICAgKX1cblxuICAgIDxzZWN0aW9uPlxuICAgICAge2dhbWVzLm1hcCgoZ2FtZSkgPT4gKFxuICAgICAgICA8R2FtZSB7Li4uZ2FtZX0gLz5cbiAgICAgICkpfVxuICAgIDwvc2VjdGlvbj5cbiAgPC9tYWluPlxuKTtcblxuY29uc3QgR2FtZSA9ICh7XG4gIG5hbWUsXG4gIGRlc2NyaXB0aW9uLFxuICB0eXBlLFxuICBza3UsXG4gIGFwcCxcbiAgYWRkb25zLFxuICBidW5kbGVzLFxuICBwcmljZXMsXG59KSA9PiAoXG4gIDxzZWN0aW9uPlxuICAgIDxoMj5cbiAgICAgIHtuYW1lfSBbXG4gICAgICB7dHlwZSA9PT0gXCJnYW1lXCIgPyAoXG4gICAgICAgIDxzcGFuPlxuICAgICAgICAgIDxhIGhyZWY9e2BodHRwOi8vc3RhZGlhLmdvb2dsZS5jb20vcGxheWVyLyR7YXBwfWB9PmxhdW5jaCBnYW1lPC9hPiBvclxuICAgICAgICA8L3NwYW4+XG4gICAgICApIDogKFxuICAgICAgICB0eXBlXG4gICAgICApfXtcIiBcIn1cbiAgICAgIDxhIGhyZWY9e2BodHRwczovL3N0YWRpYS5nb29nbGUuY29tL3N0b3JlL2RldGFpbHMvJHthcHB9L3NrdS8ke3NrdX1gfT5cbiAgICAgICAgdmlldyBpbiBzdG9yZVxuICAgICAgPC9hPntcIiBcIn1cbiAgICAgIHtQcmljZXMucHJvdG90eXBlLnJlbmRlci5jYWxsKHByaWNlcyl9XVxuICAgIDwvaDI+XG4gICAgPHA+e2Rlc2NyaXB0aW9ufTwvcD5cbiAgICB7YWRkb25zLm1hcCgoYWRkb24pID0+IChcbiAgICAgIDxzZWN0aW9uPlxuICAgICAgICA8aDM+XG4gICAgICAgICAgPHNwYW4gc3R5bGU9XCJmb250LXdlaWdodDogbm9ybWFsO1wiPuKelTwvc3Bhbj4ge2FkZG9uLm5hbWV9IFtcbiAgICAgICAgICA8YVxuICAgICAgICAgICAgaHJlZj17YGh0dHBzOi8vc3RhZGlhLmdvb2dsZS5jb20vc3RvcmUvZGV0YWlscy8ke2FwcH0vc2t1LyR7YWRkb24uc2t1fWB9XG4gICAgICAgICAgPlxuICAgICAgICAgICAgdmlldyBpbiBzdG9yZVxuICAgICAgICAgIDwvYT57XCIgXCJ9XG4gICAgICAgICAge1ByaWNlcy5wcm90b3R5cGUucmVuZGVyLmNhbGwoYWRkb24ucHJpY2VzKX1dXG4gICAgICAgIDwvaDM+XG4gICAgICAgIDxwPntkZXNjcmlwdGlvbn08L3A+XG4gICAgICA8L3NlY3Rpb24+XG4gICAgKSl9XG4gICAge2J1bmRsZXMubWFwKChidW5kbGUpID0+IChcbiAgICAgIDxzZWN0aW9uPlxuICAgICAgICA8aDM+XG4gICAgICAgICAgPHNwYW4gc3R5bGU9XCJmb250LXdlaWdodDogbm9ybWFsO1wiPvCfk6Y8L3NwYW4+IHtidW5kbGUubmFtZX0gW1xuICAgICAgICAgIDxhXG4gICAgICAgICAgICBocmVmPXtgaHR0cHM6Ly9zdGFkaWEuZ29vZ2xlLmNvbS9zdG9yZS9kZXRhaWxzLyR7YXBwfS9za3UvJHtidW5kbGUuc2t1fWB9XG4gICAgICAgICAgPlxuICAgICAgICAgICAgdmlldyBpbiBzdG9yZVxuICAgICAgICAgIDwvYT57XCIgXCJ9XG4gICAgICAgICAge1ByaWNlcy5wcm90b3R5cGUucmVuZGVyLmNhbGwoYnVuZGxlLnByaWNlcyl9XVxuICAgICAgICA8L2gzPlxuICAgICAgICA8cD57ZGVzY3JpcHRpb259PC9wPlxuICAgICAgPC9zZWN0aW9uPlxuICAgICkpfVxuICA8L3NlY3Rpb24+XG4pO1xuIl19
