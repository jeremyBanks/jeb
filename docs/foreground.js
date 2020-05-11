import "./foreground/autoreload.js";
import "./foreground/jsx.js";
import { Prices } from "./foreground/data/models.js";
import { spider } from "./foreground/spider.js";
(async () => {
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
  document.body.appendChild(JSX.createElement(Home, { games: games }));
})();
const Home = ({ games }) =>
  JSX.createElement(
    "main",
    {
      style: {
        fontSize: "14px",
        maxWidth: "800px",
        margin: "8px 16px",
        backgroundImage: "url(/illufinch-violetsky-edited@2x.png)",
        backgroundPosition: "top 16px right 16px",
        backgroundRepeat: "no-repeat",
        backgroundSize: "64px",
      },
    },
    JSX.createElement("h1", null, "stadia.observer"),
    window.chrome?.runtime?.id &&
      JSX.createElement(
        "button",
        { onclick: spider },
        "\uD83D\uDD77\uFE0F Spider Stadia Store",
      ),
    JSX.createElement(
      "section",
      { className: "gameList" },
      games.map(game => JSX.createElement(Game, Object.assign({}, game))),
    ),
  );
const Game = ({ name, description, type, sku, app, addons, bundles, prices }) =>
  JSX.createElement(
    "section",
    null,
    JSX.createElement(
      "section",
      {
        style: {
          display: "grid",
          gridTemplateColumns: "1fr 1fr 1fr",
        },
      },
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
                "launch game",
              ),
              " ",
              "or",
            )
          : type,
        " ",
        JSX.createElement(
          "a",
          { href: `https://stadia.google.com/store/details/${app}/sku/${sku}` },
          "view in store",
        ),
        " ",
        Prices.prototype.render.call(prices),
        "]",
      ),
    ),
    JSX.createElement(
      "section",
      {
        style: {
          display: "grid",
          gridTemplateColumns: "1fr 1fr 1fr",
        },
      },
      JSX.createElement("p", null, description),
      addons.map(addon =>
        JSX.createElement(
          "section",
          null,
          JSX.createElement(
            "h3",
            null,
            JSX.createElement(
              "span",
              { style: { fontWeight: "normal" } },
              "\u2795",
            ),
            " ",
            addon.name,
            " [",
            JSX.createElement(
              "a",
              {
                href: `https://stadia.google.com/store/details/${app}/sku/${addon.sku}`,
              },
              "view in store",
            ),
            " ",
            Prices.prototype.render.call(addon.prices),
            "]",
          ),
          JSX.createElement("p", null, description),
        ),
      ),
    ),
    JSX.createElement(
      "section",
      {
        style: {
          display: "grid",
          gridTemplateColumns: "1fr 1fr 1fr",
        },
      },
      bundles.map(bundle =>
        JSX.createElement(
          "section",
          null,
          JSX.createElement(
            "h3",
            null,
            JSX.createElement(
              "span",
              { style: { fontWeight: "normal" } },
              "\uD83D\uDCE6",
            ),
            " ",
            bundle.name,
            " [",
            JSX.createElement(
              "a",
              {
                href: `https://stadia.google.com/store/details/${app}/sku/${bundle.sku}`,
              },
              "view in store",
            ),
            " ",
            Prices.prototype.render.call(bundle.prices),
            "]",
          ),
          JSX.createElement("p", null, description),
        ),
      ),
    ),
  );
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiZm9yZWdyb3VuZC5qcyIsInNvdXJjZVJvb3QiOiIuLyIsInNvdXJjZXMiOlsiZm9yZWdyb3VuZC50c3giXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQUEsT0FBTyw0QkFBNEIsQ0FBQztBQUNwQyxPQUFPLHFCQUFxQixDQUFDO0FBRTdCLE9BQU8sRUFBRSxNQUFNLEVBQUUsTUFBTSw2QkFBNkIsQ0FBQztBQUNyRCxPQUFPLEVBQUUsTUFBTSxFQUFFLE1BQU0sd0JBQXdCLENBQUM7QUFFaEQsQ0FBQyxLQUFLLElBQUksRUFBRTtJQUNWLFFBQVEsQ0FBQyxLQUFLLEdBQUcsaUJBQWlCLENBQUM7SUFDbkMsUUFBUSxDQUFDLElBQUksQ0FBQyxXQUFXLENBQ3ZCLE1BQU0sQ0FBQyxNQUFNLENBQUMsUUFBUSxDQUFDLGFBQWEsQ0FBQyxNQUFNLENBQUMsRUFBRTtRQUM1QyxHQUFHLEVBQUUsTUFBTTtRQUNYLElBQUksRUFBRSxvQ0FBb0M7S0FDM0MsQ0FBQyxDQUNILENBQUM7SUFFRixNQUFNLElBQUksR0FBRyxNQUFNLENBQUMsTUFBTSxDQUFDLE1BQU0sQ0FBQyxNQUFNLEtBQUssQ0FBQyxZQUFZLENBQUMsQ0FBQyxDQUFDLElBQUksRUFBRSxDQUFDLENBQUM7SUFDckUsTUFBTSxRQUFRLEdBQUcsTUFBTSxDQUFDLFdBQVcsQ0FBQyxJQUFJLENBQUMsR0FBRyxDQUFDLEdBQUcsQ0FBQyxFQUFFLENBQUMsQ0FBQyxHQUFHLENBQUMsR0FBRyxFQUFFLEdBQUcsQ0FBQyxDQUFDLENBQUMsQ0FBQztJQUNyRSxNQUFNLGFBQWEsR0FBRyxJQUFJLENBQUMsTUFBTSxDQUFDLEdBQUcsQ0FBQyxFQUFFLENBQUMsR0FBRyxDQUFDLElBQUksS0FBSyxjQUFjLENBQUMsQ0FBQztJQUV0RSxNQUFNLEtBQUssR0FBRyxJQUFJO1NBQ2YsTUFBTSxDQUFDLEdBQUcsQ0FBQyxFQUFFLENBQUMsR0FBRyxDQUFDLElBQUksS0FBSyxNQUFNLENBQUM7U0FDbEMsR0FBRyxDQUFDLElBQUks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QU0sQ0FBQyxTQUFTLENBQUMsTUFBTSxDQUFDLElBQUksQ0FBQyxLQUFLLENBQUMsTUFBTSxDQUFDO29CQUN4QztZQUNMLDZCQUFJLFdBQVcsQ0FBSyxDQUNaLENBQ1gsQ0FBQyxDQUNNO0lBQ1YsK0JBQ0UsS0FBSyxFQUFFO1lBQ0wsT0FBTyxFQUFFLE1BQU07WUFDZixtQkFBbUIsRUFBRSxhQUFhO1NBQ25DLElBRUEsT0FBTyxDQUFDLEdBQUcsQ0FBQyxNQUFNLENBQUMsRUFBRSxDQUFDLENBQ3JCO1FBQ0U7WUFDRSw0QkFBTSxLQUFLLEVBQUUsRUFBRSxVQUFVLEVBQUUsUUFBUSxFQUFFLG1CQUFXOztZQUFFLE1BQU0sQ0FBQyxJQUFJOztZQUM3RCx5QkFDRSxJQUFJLEVBQUUsMkNBQTJDLEdBQUcsUUFBUSxNQUFNLENBQUMsR0FBRyxFQUFFLG9CQUd0RTtZQUFDLEdBQUc7WUFDUCxNQUFNLENBQUMsU0FBUyxDQUFDLE1BQU0sQ0FBQyxJQUFJLENBQUMsTUFBTSxDQUFDLE1BQU0sQ0FBQztnQkFDekM7UUFDTCw2QkFBSSxXQUFXLENBQUssQ0FDWixDQUNYLENBQUMsQ0FDTSxDQUNGLENBQ1gsQ0FBQyIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCBcIi4vZm9yZWdyb3VuZC9hdXRvcmVsb2FkLmpzXCI7XG5pbXBvcnQgXCIuL2ZvcmVncm91bmQvanN4LmpzXCI7XG5cbmltcG9ydCB7IFByaWNlcyB9IGZyb20gXCIuL2ZvcmVncm91bmQvZGF0YS9tb2RlbHMuanNcIjtcbmltcG9ydCB7IHNwaWRlciB9IGZyb20gXCIuL2ZvcmVncm91bmQvc3BpZGVyLmpzXCI7XG5cbihhc3luYyAoKSA9PiB7XG4gIGRvY3VtZW50LnRpdGxlID0gXCJzdGFkaWEub2JzZXJ2ZXJcIjtcbiAgZG9jdW1lbnQuaGVhZC5hcHBlbmRDaGlsZChcbiAgICBPYmplY3QuYXNzaWduKGRvY3VtZW50LmNyZWF0ZUVsZW1lbnQoXCJsaW5rXCIpLCB7XG4gICAgICByZWw6IFwiaWNvblwiLFxuICAgICAgaHJlZjogXCIvaWxsdWZpbmNoLXZpb2xldHNreS1lZGl0ZWRAMngucG5nXCIsXG4gICAgfSksXG4gICk7XG5cbiAgY29uc3Qgc2t1cyA9IE9iamVjdC52YWx1ZXMoYXdhaXQgKGF3YWl0IGZldGNoKFwiL3NrdXMuanNvblwiKSkuanNvbigpKTtcbiAgY29uc3Qgc2t1c0J5SWQgPSBPYmplY3QuZnJvbUVudHJpZXMoc2t1cy5tYXAoc2t1ID0+IFtza3Uuc2t1LCBza3VdKSk7XG4gIGNvbnN0IHN1YnNjcmlwdGlvbnMgPSBza3VzLmZpbHRlcihza3UgPT4gc2t1LnR5cGUgPT09IFwic3Vic2NyaXB0aW9uXCIpO1xuXG4gIGNvbnN0IGdhbWVzID0gc2t1c1xuICAgIC5maWx0ZXIoc2t1ID0+IHNrdS50eXBlID09PSBcImdhbWVcIilcbiAgICAubWFwKGdhbWUgPT5cbiAgICAgIE9iamVjdC5hc3NpZ24oZ2FtZSwge1xuICAgICAgICBza3VzOiBza3VzLmZpbHRlcihza3UgPT4gc2t1LmFwcCA9PT0gZ2FtZS5hcHApLFxuICAgICAgICBhZGRvbnM6IHNrdXMuZmlsdGVyKFxuICAgICAgICAgIHNrdSA9PiBza3UuYXBwID09PSBnYW1lLmFwcCAmJiBza3UudHlwZSA9PT0gXCJhZGRvblwiLFxuICAgICAgICApLFxuICAgICAgICBidW5kbGVzOiBza3VzLmZpbHRlcihcbiAgICAgICAgICBza3UgPT4gc2t1LmFwcCA9PT0gZ2FtZS5hcHAgJiYgc2t1LnR5cGUgPT09IFwiYnVuZGxlXCIsXG4gICAgICAgICksXG4gICAgICB9KSxcbiAgICApO1xuXG4gIGRvY3VtZW50LmJvZHkuYXBwZW5kQ2hpbGQoPEhvbWUgZ2FtZXM9e2dhbWVzfSAvPik7XG59KSgpO1xuXG5jb25zdCBIb21lID0gKHsgZ2FtZXMgfTogeyBnYW1lczogQXJyYXk8YW55PiB9KSA9PiAoXG4gIDxtYWluXG4gICAgc3R5bGU9e3tcbiAgICAgIGZvbnRTaXplOiBcIjE0cHhcIixcbiAgICAgIG1heFdpZHRoOiBcIjgwMHB4XCIsXG4gICAgICBtYXJnaW46IFwiOHB4IDE2cHhcIixcbiAgICAgIGJhY2tncm91bmRJbWFnZTogXCJ1cmwoL2lsbHVmaW5jaC12aW9sZXRza3ktZWRpdGVkQDJ4LnBuZylcIixcbiAgICAgIGJhY2tncm91bmRQb3NpdGlvbjogXCJ0b3AgMTZweCByaWdodCAxNnB4XCIsXG4gICAgICBiYWNrZ3JvdW5kUmVwZWF0OiBcIm5vLXJlcGVhdFwiLFxuICAgICAgYmFja2dyb3VuZFNpemU6IFwiNjRweFwiLFxuICAgIH19XG4gID5cbiAgICA8aDE+c3RhZGlhLm9ic2VydmVyPC9oMT5cblxuICAgIHt3aW5kb3cuY2hyb21lPy5ydW50aW1lPy5pZCAmJiAoXG4gICAgICA8YnV0dG9uIG9uY2xpY2s9e3NwaWRlcn0+8J+Vt++4jyBTcGlkZXIgU3RhZGlhIFN0b3JlPC9idXR0b24+XG4gICAgKX1cblxuICAgIDxzZWN0aW9uIGNsYXNzTmFtZT1cImdhbWVMaXN0XCI+XG4gICAgICB7Z2FtZXMubWFwKGdhbWUgPT4gKFxuICAgICAgICA8R2FtZSB7Li4uZ2FtZX0gLz5cbiAgICAgICkpfVxuICAgIDwvc2VjdGlvbj5cbiAgPC9tYWluPlxuKTtcblxuY29uc3QgR2FtZSA9ICh7XG4gIG5hbWUsXG4gIGRlc2NyaXB0aW9uLFxuICB0eXBlLFxuICBza3UsXG4gIGFwcCxcbiAgYWRkb25zLFxuICBidW5kbGVzLFxuICBwcmljZXMsXG59KSA9PiAoXG4gIDxzZWN0aW9uPlxuICAgIDxzZWN0aW9uXG4gICAgICBzdHlsZT17e1xuICAgICAgICBkaXNwbGF5OiBcImdyaWRcIixcbiAgICAgICAgZ3JpZFRlbXBsYXRlQ29sdW1uczogXCIxZnIgMWZyIDFmclwiLFxuICAgICAgfX1cbiAgICA+XG4gICAgICA8aDI+XG4gICAgICAgIHtuYW1lfSBbXG4gICAgICAgIHt0eXBlID09PSBcImdhbWVcIiA/IChcbiAgICAgICAgICA8c3Bhbj5cbiAgICAgICAgICAgIDxhIGhyZWY9e2BodHRwOi8vc3RhZGlhLmdvb2dsZS5jb20vcGxheWVyLyR7YXBwfWB9PmxhdW5jaCBnYW1lPC9hPntcIiBcIn1cbiAgICAgICAgICAgIG9yXG4gICAgICAgICAgPC9zcGFuPlxuICAgICAgICApIDogKFxuICAgICAgICAgIHR5cGVcbiAgICAgICAgKX17XCIgXCJ9XG4gICAgICAgIDxhIGhyZWY9e2BodHRwczovL3N0YWRpYS5nb29nbGUuY29tL3N0b3JlL2RldGFpbHMvJHthcHB9L3NrdS8ke3NrdX1gfT5cbiAgICAgICAgICB2aWV3IGluIHN0b3JlXG4gICAgICAgIDwvYT57XCIgXCJ9XG4gICAgICAgIHtQcmljZXMucHJvdG90eXBlLnJlbmRlci5jYWxsKHByaWNlcyl9XVxuICAgICAgPC9oMj5cbiAgICA8L3NlY3Rpb24+XG4gICAgPHNlY3Rpb25cbiAgICAgIHN0eWxlPXt7XG4gICAgICAgIGRpc3BsYXk6IFwiZ3JpZFwiLFxuICAgICAgICBncmlkVGVtcGxhdGVDb2x1bW5zOiBcIjFmciAxZnIgMWZyXCIsXG4gICAgICB9fVxuICAgID5cbiAgICAgIDxwPntkZXNjcmlwdGlvbn08L3A+XG4gICAgICB7YWRkb25zLm1hcChhZGRvbiA9PiAoXG4gICAgICAgIDxzZWN0aW9uPlxuICAgICAgICAgIDxoMz5cbiAgICAgICAgICAgIDxzcGFuIHN0eWxlPXt7IGZvbnRXZWlnaHQ6IFwibm9ybWFsXCIgfX0+4p6VPC9zcGFuPiB7YWRkb24ubmFtZX0gW1xuICAgICAgICAgICAgPGFcbiAgICAgICAgICAgICAgaHJlZj17YGh0dHBzOi8vc3RhZGlhLmdvb2dsZS5jb20vc3RvcmUvZGV0YWlscy8ke2FwcH0vc2t1LyR7YWRkb24uc2t1fWB9XG4gICAgICAgICAgICA+XG4gICAgICAgICAgICAgIHZpZXcgaW4gc3RvcmVcbiAgICAgICAgICAgIDwvYT57XCIgXCJ9XG4gICAgICAgICAgICB7UHJpY2VzLnByb3RvdHlwZS5yZW5kZXIuY2FsbChhZGRvbi5wcmljZXMpfV1cbiAgICAgICAgICA8L2gzPlxuICAgICAgICAgIDxwPntkZXNjcmlwdGlvbn08L3A+XG4gICAgICAgIDwvc2VjdGlvbj5cbiAgICAgICkpfVxuICAgIDwvc2VjdGlvbj5cbiAgICA8c2VjdGlvblxuICAgICAgc3R5bGU9e3tcbiAgICAgICAgZGlzcGxheTogXCJncmlkXCIsXG4gICAgICAgIGdyaWRUZW1wbGF0ZUNvbHVtbnM6IFwiMWZyIDFmciAxZnJcIixcbiAgICAgIH19XG4gICAgPlxuICAgICAge2J1bmRsZXMubWFwKGJ1bmRsZSA9PiAoXG4gICAgICAgIDxzZWN0aW9uPlxuICAgICAgICAgIDxoMz5cbiAgICAgICAgICAgIDxzcGFuIHN0eWxlPXt7IGZvbnRXZWlnaHQ6IFwibm9ybWFsXCIgfX0+8J+Tpjwvc3Bhbj4ge2J1bmRsZS5uYW1lfSBbXG4gICAgICAgICAgICA8YVxuICAgICAgICAgICAgICBocmVmPXtgaHR0cHM6Ly9zdGFkaWEuZ29vZ2xlLmNvbS9zdG9yZS9kZXRhaWxzLyR7YXBwfS9za3UvJHtidW5kbGUuc2t1fWB9XG4gICAgICAgICAgICA+XG4gICAgICAgICAgICAgIHZpZXcgaW4gc3RvcmVcbiAgICAgICAgICAgIDwvYT57XCIgXCJ9XG4gICAgICAgICAgICB7UHJpY2VzLnByb3RvdHlwZS5yZW5kZXIuY2FsbChidW5kbGUucHJpY2VzKX1dXG4gICAgICAgICAgPC9oMz5cbiAgICAgICAgICA8cD57ZGVzY3JpcHRpb259PC9wPlxuICAgICAgICA8L3NlY3Rpb24+XG4gICAgICApKX1cbiAgICA8L3NlY3Rpb24+XG4gIDwvc2VjdGlvbj5cbik7XG4iXX0=
