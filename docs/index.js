import { Prices } from "./index/data/models.js";
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
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiaW5kZXguanMiLCJzb3VyY2VSb290IjoiLi8iLCJzb3VyY2VzIjpbImluZGV4LnRzeCJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLEVBQUUsTUFBTSxFQUFFLE1BQU0sd0JBQXdCLENBQUM7QUFDaEQsT0FBTyxFQUFFLE1BQU0sRUFBRSxNQUFNLG1CQUFtQixDQUFDO0FBQzNDLE9BQU8sRUFBRSxNQUFNLEVBQUUsTUFBTSxtQkFBbUIsQ0FBQztBQUUzQyxDQUFDLEtBQUssSUFBSSxFQUFFO0lBQ1YsUUFBUSxDQUFDLEtBQUssR0FBRyxpQkFBaUIsQ0FBQztJQUNuQyxRQUFRLENBQUMsSUFBSSxDQUFDLFdBQVcsQ0FDdkIsTUFBTSxDQUFDLE1BQU0sQ0FBQyxRQUFRLENBQUMsYUFBYSxDQUFDLE1BQU0sQ0FBQyxFQUFFO1FBQzVDLEdBQUcsRUFBRSxNQUFNO1FBQ1gsSUFBSSxFQUFFLG9DQUFvQztLQUMzQyxDQUFDLENBQ0gsQ0FBQztJQUVGLE1BQU0sSUFBSSxHQUFHLE1BQU0sQ0FBQyxNQUFNLENBQUMsTUFBTSxDQUFDLE1BQU0sS0FBSyxDQUFDLFlBQVksQ0FBQyxDQUFDLENBQUMsSUFBSSxFQUFFLENBQUMsQ0FBQztJQUNyRSxNQUFNLFFBQVEsR0FBRyxNQUFNLENBQUMsV0FBVyxDQUFDLElBQUksQ0FBQyxHQUFHLENBQUMsQ0FBQyxHQUFHLEVBQUUsRUFBRSxDQUFDLENBQUMsR0FBRyxDQUFDLEdBQUcsRUFBRSxHQUFHLENBQUMsQ0FBQyxDQUFDLENBQUM7SUFDdkUsTUFBTSxhQUFhLEdBQUcsSUFBSSxDQUFDLE1BQU0sQ0FBQyxDQUFDLEdBQUcsRUFBRSxFQUFFLENBQUMsR0FBRyxDQUFDLElBQUksS0FBSyxjQUFjLENBQUMsQ0FBQztJQUV4RSxNQUFNLEtBQUssR0FBRyxJQUFJO1NBQ2YsTUFBTSxDQUFDLENBQUMsR0FBRyxFQUFFLEVBQUUsQ0FBQyxHQUFHLENBQUMsSUFBSSxLQUFLLE1BQU0sQ0FBQztTQUNwQyxHQUFHLENBQUMsQ0FB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TSxDQUFDLElBQUksQ0FBQyxNQUFNLENBQUMsTUFBTSxDQUFDO2dCQUN6QztRQUNMLGtCQUFJLFdBQVcsQ0FBSyxDQUNaLENBQ1gsQ0FBQyxDQUNNLENBQ1gsQ0FBQyIsInNvdXJjZXNDb250ZW50IjpbImltcG9ydCB7IFByaWNlcyB9IGZyb20gXCIuL2luZGV4L2RhdGEvbW9kZWxzLmpzXCI7XG5pbXBvcnQgeyByZW5kZXIgfSBmcm9tIFwiLi9pbmRleC9yZW5kZXIuanNcIjtcbmltcG9ydCB7IHNwaWRlciB9IGZyb20gXCIuL2luZGV4L3NwaWRlci5qc1wiO1xuXG4oYXN5bmMgKCkgPT4ge1xuICBkb2N1bWVudC50aXRsZSA9IFwic3RhZGlhLm9ic2VydmVyXCI7XG4gIGRvY3VtZW50LmhlYWQuYXBwZW5kQ2hpbGQoXG4gICAgT2JqZWN0LmFzc2lnbihkb2N1bWVudC5jcmVhdGVFbGVtZW50KFwibGlua1wiKSwge1xuICAgICAgcmVsOiBcImljb25cIixcbiAgICAgIGhyZWY6IFwiL2lsbHVmaW5jaC12aW9sZXRza3ktZWRpdGVkQDJ4LnBuZ1wiLFxuICAgIH0pXG4gICk7XG5cbiAgY29uc3Qgc2t1cyA9IE9iamVjdC52YWx1ZXMoYXdhaXQgKGF3YWl0IGZldGNoKFwiL3NrdXMuanNvblwiKSkuanNvbigpKTtcbiAgY29uc3Qgc2t1c0J5SWQgPSBPYmplY3QuZnJvbUVudHJpZXMoc2t1cy5tYXAoKHNrdSkgPT4gW3NrdS5za3UsIHNrdV0pKTtcbiAgY29uc3Qgc3Vic2NyaXB0aW9ucyA9IHNrdXMuZmlsdGVyKChza3UpID0+IHNrdS50eXBlID09PSBcInN1YnNjcmlwdGlvblwiKTtcblxuICBjb25zdCBnYW1lcyA9IHNrdXNcbiAgICAuZmlsdGVyKChza3UpID0+IHNrdS50eXBlID09PSBcImdhbWVcIilcbiAgICAubWFwKChnYW1lKSA9PlxuICAgICAgT2JqZWN0LmFzc2lnbihnYW1lLCB7XG4gICAgICAgIHNrdXM6IHNrdXMuZmlsdGVyKChza3UpID0+IHNrdS5hcHAgPT09IGdhbWUuYXBwKSxcbiAgICAgICAgYWRkb25zOiBza3VzLmZpbHRlcihcbiAgICAgICAgICAoc2t1KSA9PiBza3UuYXBwID09PSBnYW1lLmFwcCAmJiBza3UudHlwZSA9PT0gXCJhZGRvblwiXG4gICAgICAgICksXG4gICAgICAgIGJ1bmRsZXM6IHNrdXMuZmlsdGVyKFxuICAgICAgICAgIChza3UpID0+IHNrdS5hcHAgPT09IGdhbWUuYXBwICYmIHNrdS50eXBlID09PSBcImJ1bmRsZVwiXG4gICAgICAgICksXG4gICAgICB9KVxuICAgICk7XG5cbiAgZG9jdW1lbnQuYm9keS5hcHBlbmRDaGlsZCg8SG9tZSBnYW1lcz17Z2FtZXN9IC8+KTtcbn0pKCk7XG5cbmNvbnN0IEhvbWUgPSAoeyBnYW1lcyB9OiBhbnkpID0+IChcbiAgPG1haW5cbiAgICBzdHlsZT17YFxuICAgIGZvbnQtc2l6ZTogMTRweDtcbiAgICBtYXgtd2lkdGg6IDgwMHB4O1xuICAgIG1hcmdpbjogOHB4IDE2cHg7XG5cbiAgICBiYWNrZ3JvdW5kLWltYWdlOiB1cmwoL2lsbHVmaW5jaC12aW9sZXRza3ktZWRpdGVkQDJ4LnBuZyk7XG4gICAgYmFja2dyb3VuZC1wb3NpdGlvbjogdG9wIDE2cHggcmlnaHQgMTZweDtcbiAgICBiYWNrZ3JvdW5kLXJlcGVhdDogbm8tcmVwZWF0O1xuICAgIGJhY2tncm91bmQtc2l6ZTogNjRweDtcbiAgYH1cbiAgPlxuICAgIDxoMT5zdGFkaWEub2JzZXJ2ZXI8L2gxPlxuXG4gICAge3dpbmRvdy5jaHJvbWU/LnJ1bnRpbWU/LmlkICYmIChcbiAgICAgIDxidXR0b24gb25jbGljaz17c3BpZGVyfT7wn5W377iPc3BpZGVyIHN0YWRpYTwvYnV0dG9uPlxuICAgICl9XG5cbiAgICA8c2VjdGlvbj5cbiAgICAgIHtnYW1lcy5tYXAoKGdhbWUpID0+IChcbiAgICAgICAgPEdhbWUgey4uLmdhbWV9IC8+XG4gICAgICApKX1cbiAgICA8L3NlY3Rpb24+XG4gIDwvbWFpbj5cbik7XG5cbmNvbnN0IEdhbWUgPSAoe1xuICBuYW1lLFxuICBkZXNjcmlwdGlvbixcbiAgdHlwZSxcbiAgc2t1LFxuICBhcHAsXG4gIGFkZG9ucyxcbiAgYnVuZGxlcyxcbiAgcHJpY2VzLFxufSkgPT4gKFxuICA8c2VjdGlvbj5cbiAgICA8aDI+XG4gICAgICB7bmFtZX0gW1xuICAgICAge3R5cGUgPT09IFwiZ2FtZVwiID8gKFxuICAgICAgICA8c3Bhbj5cbiAgICAgICAgICA8YSBocmVmPXtgaHR0cDovL3N0YWRpYS5nb29nbGUuY29tL3BsYXllci8ke2FwcH1gfT5sYXVuY2ggZ2FtZTwvYT4gb3JcbiAgICAgICAgPC9zcGFuPlxuICAgICAgKSA6IChcbiAgICAgICAgdHlwZVxuICAgICAgKX17XCIgXCJ9XG4gICAgICA8YSBocmVmPXtgaHR0cHM6Ly9zdGFkaWEuZ29vZ2xlLmNvbS9zdG9yZS9kZXRhaWxzLyR7YXBwfS9za3UvJHtza3V9YH0+XG4gICAgICAgIHZpZXcgaW4gc3RvcmVcbiAgICAgIDwvYT57XCIgXCJ9XG4gICAgICB7UHJpY2VzLnByb3RvdHlwZS5yZW5kZXIuY2FsbChwcmljZXMpfV1cbiAgICA8L2gyPlxuICAgIDxwPntkZXNjcmlwdGlvbn08L3A+XG4gICAge2FkZG9ucy5tYXAoKGFkZG9uKSA9PiAoXG4gICAgICA8c2VjdGlvbj5cbiAgICAgICAgPGgzPlxuICAgICAgICAgIDxzcGFuIHN0eWxlPVwiZm9udC13ZWlnaHQ6IG5vcm1hbDtcIj7inpU8L3NwYW4+IHthZGRvbi5uYW1lfSBbXG4gICAgICAgICAgPGFcbiAgICAgICAgICAgIGhyZWY9e2BodHRwczovL3N0YWRpYS5nb29nbGUuY29tL3N0b3JlL2RldGFpbHMvJHthcHB9L3NrdS8ke2FkZG9uLnNrdX1gfVxuICAgICAgICAgID5cbiAgICAgICAgICAgIHZpZXcgaW4gc3RvcmVcbiAgICAgICAgICA8L2E+e1wiIFwifVxuICAgICAgICAgIHtQcmljZXMucHJvdG90eXBlLnJlbmRlci5jYWxsKGFkZG9uLnByaWNlcyl9XVxuICAgICAgICA8L2gzPlxuICAgICAgICA8cD57ZGVzY3JpcHRpb259PC9wPlxuICAgICAgPC9zZWN0aW9uPlxuICAgICkpfVxuICAgIHtidW5kbGVzLm1hcCgoYnVuZGxlKSA9PiAoXG4gICAgICA8c2VjdGlvbj5cbiAgICAgICAgPGgzPlxuICAgICAgICAgIDxzcGFuIHN0eWxlPVwiZm9udC13ZWlnaHQ6IG5vcm1hbDtcIj7wn5OmPC9zcGFuPiB7YnVuZGxlLm5hbWV9IFtcbiAgICAgICAgICA8YVxuICAgICAgICAgICAgaHJlZj17YGh0dHBzOi8vc3RhZGlhLmdvb2dsZS5jb20vc3RvcmUvZGV0YWlscy8ke2FwcH0vc2t1LyR7YnVuZGxlLnNrdX1gfVxuICAgICAgICAgID5cbiAgICAgICAgICAgIHZpZXcgaW4gc3RvcmVcbiAgICAgICAgICA8L2E+e1wiIFwifVxuICAgICAgICAgIHtQcmljZXMucHJvdG90eXBlLnJlbmRlci5jYWxsKGJ1bmRsZS5wcmljZXMpfV1cbiAgICAgICAgPC9oMz5cbiAgICAgICAgPHA+e2Rlc2NyaXB0aW9ufTwvcD5cbiAgICAgIDwvc2VjdGlvbj5cbiAgICApKX1cbiAgPC9zZWN0aW9uPlxuKTtcbiJdfQ==
