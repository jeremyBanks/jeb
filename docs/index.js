import { Prices } from "./index/data/models.js";
import { render } from "./index/render.js";
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
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoiaW5kZXguanMiLCJzb3VyY2VSb290IjoiLi8iLCJzb3VyY2VzIjpbImluZGV4LnRzeCJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLEVBQUUsTUFBTSxFQUFFLE1BQU0sd0JBQXdCLENBQUM7QUFDaEQsT0FBTyxFQUFFLE1BQU0sRUFBRSxNQUFNLG1CQUFtQixDQUFDO0FBQzNDLE9BQU8sRUFBRSxNQUFNLEVBQUUsTUFBTSxtQkFBbUIsQ0FBQztBQUUzQyxDQUFDLEtBQUssSUFBSSxFQUFFO0lBQ1YsSUFBSSxzQkFBc0IsR0FBRyxNQUFNLENBQ2pDLE1BQU0sS0FBSyxDQUFDLDRCQUE0QixDQUFDLENBQzFDLENBQUMsSUFBSSxFQUFFLENBQUM7SUFDVCxXQUFXLENBQUMsS0FBSyxJQUFJLEVBQUU7UUFDckIsSUFBSSxnQkFBZ0IsR0FBRyxNQUFNLENBQzNCLE1BQU0sS0FBSyxDQUFDLDRCQUE0QixDQUFDLENBQzFDLENBQUMsSUFBSSxFQUFFLENBQUM7UUFDVCxJQUFJLGdCQUFnQixLQUFLLHNCQUFzQixFQUFFO1lBQy9DLFFBQVEsQ0FBQyxRQUFRLENBQUMsTUFBTSxFQUFFLENBQUM7U0FDNUI7SUFDSCxDQUFDLEVBQUUsSUFBSSxDQUFDLENBQUM7QUFDWCxDQUFDLENBQUMsRUFBRSxDQUFDO0FBRUwsQ0FBQyxLQUFLLElBQUksRUFBRTtJQUNWLFFBQVEsQ0FBQyxLQUFLLEdBQUcsaUJBQWlCLENBQUM7SUFDbkMsUUFBUSxDQUFDLElBQUksQ0FBQyxXQUFXLENBQ3ZCLE1BQU0sQ0FBQyxNQUFNLENBQUMsUUFBUSxDQUFDLGFBQWEsQ0FBQyxNQUFNLENBQUMsRUFBRTtRQUM1QyxHQUFHLEVBQUUsTUFBTTtRQUNYLElBQUksRUFBRSxvQ0FBb0M7S0FDM0MsQ0FBQyxDQUNILENBQUM7SUFFRixNQUFNLElBQUksR0FBRyxNQUFNLENBQUMsTUFBTSxDQUFDLE1BQU0sQ0FBQyxNQUFNLEtBQUssQ0FBQyxZQUFZLENBQUMsQ0FBQyxDQUFDLElBQUksRUFB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LEVBQUUsMkNBQTJDLEdBQUcsUUFBUSxLQUFLLENBQUMsR0FBRyxFQUFFLG9CQUdyRTtZQUFDLEdBQUc7WUFDUCxNQUFNLENBQUMsU0FBUyxDQUFDLE1BQU0sQ0FBQyxJQUFJLENBQUMsS0FBSyxDQUFDLE1BQU0sQ0FBQztnQkFDeEM7UUFDTCxrQkFBSSxXQUFXLENBQUssQ0FDWixDQUNYLENBQUM7SUFDRCxPQUFPLENBQUMsR0FBRyxDQUFDLENBQUMsTUFBTSxFQUFFLEVBQUUsQ0FBQyxDQUN2QjtRQUNFO1lBQ0UsaUJBQU0sS0FBSyxFQUFDLHNCQUFzQixtQkFBVTs7WUFBRSxNQUFNLENBQUMsSUFBSTs7WUFDekQsY0FDRSxJQUFJLEVBQUUsMkNBQTJDLEdBQUcsUUFBUSxNQUFNLENBQUMsR0FBRyxFQUFFLG9CQUd0RTtZQUFDLEdBQUc7WUFDUCxNQUFNLENBQUMsU0FBUyxDQUFDLE1BQU0sQ0FBQyxJQUFJLENBQUMsTUFBTSxDQUFDLE1BQU0sQ0FBQztnQkFDekM7UUFDTCxrQkFBSSxXQUFXLENBQUssQ0FDWixDQUNYLENBQUMsQ0FDTSxDQUNYLENBQUMiLCJzb3VyY2VzQ29udGVudCI6WyJpbXBvcnQgeyBQcmljZXMgfSBmcm9tIFwiLi9pbmRleC9kYXRhL21vZGVscy5qc1wiO1xuaW1wb3J0IHsgcmVuZGVyIH0gZnJvbSBcIi4vaW5kZXgvcmVuZGVyLmpzXCI7XG5pbXBvcnQgeyBzcGlkZXIgfSBmcm9tIFwiLi9pbmRleC9zcGlkZXIuanNcIjtcblxuKGFzeW5jICgpID0+IHtcbiAgbGV0IG9yaWdpbmFsQnVpbGRUaW1lc3RhbXAgPSBhd2FpdCAoXG4gICAgYXdhaXQgZmV0Y2goXCIvbGFzdC1idWlsZC10aW1lc3RhbXAuanNvblwiKVxuICApLmpzb24oKTtcbiAgc2V0SW50ZXJ2YWwoYXN5bmMgKCkgPT4ge1xuICAgIGxldCBjdXJyZW50VGltZXN0YW1wID0gYXdhaXQgKFxuICAgICAgYXdhaXQgZmV0Y2goXCIvbGFzdC1idWlsZC10aW1lc3RhbXAuanNvblwiKVxuICAgICkuanNvbigpO1xuICAgIGlmIChjdXJyZW50VGltZXN0YW1wICE9PSBvcmlnaW5hbEJ1aWxkVGltZXN0YW1wKSB7XG4gICAgICBkb2N1bWVudC5sb2NhdGlvbi5yZWxvYWQoKTtcbiAgICB9XG4gIH0sIDEwMDApO1xufSkoKTtcblxuKGFzeW5jICgpID0+IHtcbiAgZG9jdW1lbnQudGl0bGUgPSBcInN0YWRpYS5vYnNlcnZlclwiO1xuICBkb2N1bWVudC5oZWFkLmFwcGVuZENoaWxkKFxuICAgIE9iamVjdC5hc3NpZ24oZG9jdW1lbnQuY3JlYXRlRWxlbWVudChcImxpbmtcIiksIHtcbiAgICAgIHJlbDogXCJpY29uXCIsXG4gICAgICBocmVmOiBcIi9pbGx1ZmluY2gtdmlvbGV0c2t5LWVkaXRlZEAyeC5wbmdcIixcbiAgICB9KVxuICApO1xuXG4gIGNvbnN0IHNrdXMgPSBPYmplY3QudmFsdWVzKGF3YWl0IChhd2FpdCBmZXRjaChcIi9za3VzLmpzb25cIikpLmpzb24oKSk7XG4gIGNvbnN0IHNrdXNCeUlkID0gT2JqZWN0LmZyb21FbnRyaWVzKHNrdXMubWFwKChza3UpID0+IFtza3Uuc2t1LCBza3VdKSk7XG4gIGNvbnN0IHN1YnNjcmlwdGlvbnMgPSBza3VzLmZpbHRlcigoc2t1KSA9PiBza3UudHlwZSA9PT0gXCJzdWJzY3JpcHRpb25cIik7XG5cbiAgY29uc3QgZ2FtZXMgPSBza3VzXG4gICAgLmZpbHRlcigoc2t1KSA9PiBza3UudHlwZSA9PT0gXCJnYW1lXCIpXG4gICAgLm1hcCgoZ2FtZSkgPT5cbiAgICAgIE9iamVjdC5hc3NpZ24oZ2FtZSwge1xuICAgICAgICBza3VzOiBza3VzLmZpbHRlcigoc2t1KSA9PiBza3UuYXBwID09PSBnYW1lLmFwcCksXG4gICAgICAgIGFkZG9uczogc2t1cy5maWx0ZXIoXG4gICAgICAgICAgKHNrdSkgPT4gc2t1LmFwcCA9PT0gZ2FtZS5hcHAgJiYgc2t1LnR5cGUgPT09IFwiYWRkb25cIlxuICAgICAgICApLFxuICAgICAgICBidW5kbGVzOiBza3VzLmZpbHRlcihcbiAgICAgICAgICAoc2t1KSA9PiBza3UuYXBwID09PSBnYW1lLmFwcCAmJiBza3UudHlwZSA9PT0gXCJidW5kbGVcIlxuICAgICAgICApLFxuICAgICAgfSlcbiAgICApO1xuXG4gIGRvY3VtZW50LmJvZHkuYXBwZW5kQ2hpbGQoPEhvbWUgZ2FtZXM9e2dhbWVzfSAvPik7XG59KSgpO1xuXG5jb25zdCBIb21lID0gKHsgZ2FtZXMgfTogYW55KSA9PiAoXG4gIDxtYWluXG4gICAgc3R5bGU9e2BcbiAgICBmb250LXNpemU6IDE0cHg7XG4gICAgbWF4LXdpZHRoOiA4MDBweDtcbiAgICBtYXJnaW46IDhweCAxNnB4O1xuXG4gICAgYmFja2dyb3VuZC1pbWFnZTogdXJsKC9pbGx1ZmluY2gtdmlvbGV0c2t5LWVkaXRlZEAyeC5wbmcpO1xuICAgIGJhY2tncm91bmQtcG9zaXRpb246IHRvcCAxNnB4IHJpZ2h0IDE2cHg7XG4gICAgYmFja2dyb3VuZC1yZXBlYXQ6IG5vLXJlcGVhdDtcbiAgICBiYWNrZ3JvdW5kLXNpemU6IDY0cHg7XG4gIGB9XG4gID5cbiAgICA8aDE+c3RhZGlhLm9ic2VydmVyPC9oMT5cblxuICAgIDxwPmRvZXMgdGhpcyB3b3JrPzwvcD5cblxuICAgIDxociAvPlxuXG4gICAge3dpbmRvdy5jaHJvbWU/LnJ1bnRpbWU/LmlkICYmIChcbiAgICAgIDxidXR0b24gb25jbGljaz17c3BpZGVyfT7wn5W377iPc3BpZGVyIHN0YWRpYTwvYnV0dG9uPlxuICAgICl9XG5cbiAgICA8c2VjdGlvbj5cbiAgICAgIHtnYW1lcy5tYXAoKGdhbWUpID0+IChcbiAgICAgICAgPEdhbWUgey4uLmdhbWV9IC8+XG4gICAgICApKX1cbiAgICA8L3NlY3Rpb24+XG4gIDwvbWFpbj5cbik7XG5cbmNvbnN0IEdhbWUgPSAoe1xuICBuYW1lLFxuICBkZXNjcmlwdGlvbixcbiAgdHlwZSxcbiAgc2t1LFxuICBhcHAsXG4gIGFkZG9ucyxcbiAgYnVuZGxlcyxcbiAgcHJpY2VzLFxufSkgPT4gKFxuICA8c2VjdGlvbj5cbiAgICA8aDI+XG4gICAgICB7bmFtZX0gW1xuICAgICAge3R5cGUgPT09IFwiZ2FtZVwiID8gKFxuICAgICAgICA8c3Bhbj5cbiAgICAgICAgICA8YSBocmVmPXtgaHR0cDovL3N0YWRpYS5nb29nbGUuY29tL3BsYXllci8ke2FwcH1gfT5sYXVuY2ggZ2FtZTwvYT4gb3JcbiAgICAgICAgPC9zcGFuPlxuICAgICAgKSA6IChcbiAgICAgICAgdHlwZVxuICAgICAgKX17XCIgXCJ9XG4gICAgICA8YSBocmVmPXtgaHR0cHM6Ly9zdGFkaWEuZ29vZ2xlLmNvbS9zdG9yZS9kZXRhaWxzLyR7YXBwfS9za3UvJHtza3V9YH0+XG4gICAgICAgIHZpZXcgaW4gc3RvcmVcbiAgICAgIDwvYT57XCIgXCJ9XG4gICAgICB7UHJpY2VzLnByb3RvdHlwZS5yZW5kZXIuY2FsbChwcmljZXMpfV1cbiAgICA8L2gyPlxuICAgIDxwPntkZXNjcmlwdGlvbn08L3A+XG4gICAge2FkZG9ucy5tYXAoKGFkZG9uKSA9PiAoXG4gICAgICA8c2VjdGlvbj5cbiAgICAgICAgPGgzPlxuICAgICAgICAgIDxzcGFuIHN0eWxlPVwiZm9udC13ZWlnaHQ6IG5vcm1hbDtcIj7inpU8L3NwYW4+IHthZGRvbi5uYW1lfSBbXG4gICAgICAgICAgPGFcbiAgICAgICAgICAgIGhyZWY9e2BodHRwczovL3N0YWRpYS5nb29nbGUuY29tL3N0b3JlL2RldGFpbHMvJHthcHB9L3NrdS8ke2FkZG9uLnNrdX1gfVxuICAgICAgICAgID5cbiAgICAgICAgICAgIHZpZXcgaW4gc3RvcmVcbiAgICAgICAgICA8L2E+e1wiIFwifVxuICAgICAgICAgIHtQcmljZXMucHJvdG90eXBlLnJlbmRlci5jYWxsKGFkZG9uLnByaWNlcyl9XVxuICAgICAgICA8L2gzPlxuICAgICAgICA8cD57ZGVzY3JpcHRpb259PC9wPlxuICAgICAgPC9zZWN0aW9uPlxuICAgICkpfVxuICAgIHtidW5kbGVzLm1hcCgoYnVuZGxlKSA9PiAoXG4gICAgICA8c2VjdGlvbj5cbiAgICAgICAgPGgzPlxuICAgICAgICAgIDxzcGFuIHN0eWxlPVwiZm9udC13ZWlnaHQ6IG5vcm1hbDtcIj7wn5OmPC9zcGFuPiB7YnVuZGxlLm5hbWV9IFtcbiAgICAgICAgICA8YVxuICAgICAgICAgICAgaHJlZj17YGh0dHBzOi8vc3RhZGlhLmdvb2dsZS5jb20vc3RvcmUvZGV0YWlscy8ke2FwcH0vc2t1LyR7YnVuZGxlLnNrdX1gfVxuICAgICAgICAgID5cbiAgICAgICAgICAgIHZpZXcgaW4gc3RvcmVcbiAgICAgICAgICA8L2E+e1wiIFwifVxuICAgICAgICAgIHtQcmljZXMucHJvdG90eXBlLnJlbmRlci5jYWxsKGJ1bmRsZS5wcmljZXMpfV1cbiAgICAgICAgPC9oMz5cbiAgICAgICAgPHA+e2Rlc2NyaXB0aW9ufTwvcD5cbiAgICAgIDwvc2VjdGlvbj5cbiAgICApKX1cbiAgPC9zZWN0aW9uPlxuKTtcbiJdfQ==
