import * as models from "./data/models.js";
export const Home = ({ skus }) =>
  JSX.createElement(
    "main",
    null,
    JSX.createElement("h1", null, "stadia.observer"),
    JSX.createElement("hr", null),
    JSX.createElement(
      SkuList,
      null,
      skus.map(sku => JSX.createElement(Sku, Object.assign({}, sku))),
    ),
  );
Home.style = {
  fontSize: "14px",
  maxWidth: "800px",
  margin: "8px 16px",
  backgroundImage: "url(/illufinch-violetsky-edited@2x.png)",
  backgroundPosition: "top 16px right 16px",
  backgroundRepeat: "no-repeat",
  backgroundSize: "64px",
};
const SkuList = ({ children }) => JSX.createElement("section", null, children);
const Sku = sku =>
  JSX.createElement(
    "section",
    null,
    JSX.createElement(Title, { name: sku.name }),
    JSX.createElement(Description, { body: sku.description }),
    JSX.createElement(Links, { sku: sku }),
    JSX.createElement(Prices, { sku: sku }),
  );
Sku.style = {
  display: "grid",
  gridTemplateColumns: `auto 200px 100px`,
  gridTemplateAreas: `
    "Title        Links        Prices       "
    "Description  Description  Description  "
  `,
};
const Title = ({ name }) => JSX.createElement("h2", null, name);
const Description = ({ body }) => JSX.createElement("p", null, body);
const Links = ({ sku: { type, app, sku } }) =>
  JSX.createElement(
    "div",
    null,
    type === "game"
      ? JSX.createElement(
          "span",
          null,
          JSX.createElement(
            "a",
            { href: `http://stadia.google.com/player/${app}` },
            "launch game",
          ),
          " or",
          " ",
        )
      : null,
    JSX.createElement(
      "a",
      { href: `https://stadia.google.com/store/details/${app}/sku/${sku}` },
      "view in store",
    ),
  );
const Prices = ({ sku: { prices } }) =>
  JSX.createElement("div", null, models.Prices.prototype.render.call(prices));
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoidmlld3MuanMiLCJzb3VyY2VSb290IjoiLi8iLCJzb3VyY2VzIjpbImZvcmVncm91bmQvdmlld3MudHN4Il0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE9BQU8sS0FBSyxNQUFNLE1BQU0sa0JBQWtCLENBQUM7QUFFM0MsTUFBTSxDQUFDLE1BQU0sSUFBSSxHQUF3QyxDQUFDLEVBQUUsSUFBSSxFQUFFLEVBQUUsRUFBRSxDQUFDLENBQ3JFO0lBQ0UsZ0RBQXdCO0lBRXhCLDZCQUFNO0lBRU4sa0JBQUMsT0FBTyxRQUNMLElBQUksQ0FBQyxHQUFHLENBQUMsR0FBRyxDQUFDLEVBQUUsQ0FBQyxDQUNmLGtCQUFDLEdBQUcsb0JBQUssR0FBRyxFQUFJLENBQ2pCLENBQUMsQ0FDTSxDQUNMLENBQ1IsQ0FBQztBQUVGLElBQUksQ0FBQyxLQUFLLEdBQUc7SUFDWCxRQUFRLEVBQUUsTUFBTTtJQUNoQixRQUFRLEVBQUUsT0FBTztJQUNqQixNQUFNLEVBQUUsVUFBVTtJQUNsQixlQUFlLEVBQUUseUNBQXlDO0lBQzFELGtCQUFrQixFQUFFLHFCQUFxQjtJQUN6QyxnQkFBZ0IsRUFBRSxXQUFXO0lBQzdCLGNBQWMsRUFBRSxNQUFNO0NBQ3ZCLENBQUM7QUFFRixNQUFNLE9BQU8sR0FBeUMsQ0FBQyxFQUFFLFFBQVEsRUFBRSxFQUFFLEVBQUUsQ0FBQyxDQUN0RSxtQ0FBVSxRQUFRLENBQVcsQ0FDOUIsQ0FBQztBQUVGLE1BQU0sR0FBRyxHQUF1QixHQUFHLENBQUMsRUFBRSxDQUFDLENBQ3JDO0lBQ0Usa0JBQUMsS0FBSyxJQUFDLElBQUksRUFBRSxHQUFHLENBQUMsSUFBSSxHQUFJO0lBQ3pCLGtCQUFDLFdBQVcsSUFBQyxJQUFJLEVBQUUsR0FBRyxDQUFDLFdBQVcsR0FBSTtJQUN0QyxrQkFBQyxLQUFLLElBQUMsR0FBRyxFQUFFLEdBQUcsR0FBSTtJQUNuQixrQkFBQyxNQUFNLElBQUMsR0FBRyxFQUFFLEdBQUcsR0FBSSxDQUNaLENBQ1gsQ0FBQztBQUVGLEdBQUcsQ0FBQyxLQUFLLEdBQUc7SUFDVixPQUFPLEVBQUUsTUFBTTtJQUNmLG1CQUFtQixFQUFFLGtCQUFrQjtJQUN2QyxpQkFBaUIsRUFBRTs7O0dBR2xCO0NBQ0YsQ0FBQztBQUVGLE1BQU0sS0FBSyxHQUE2QixDQUFDLEVBQUUsSUFBSSxFQUFFLEVBQUUsRUFBRSxDQUFDLDhCQUFLLElBQUksQ0FBTSxDQUFDO0FBRXRFLE1BQU0sV0FBVyxHQUE2QixDQUFDLEVBQUUsSUFBSSxFQUFFLEVBQUUsRUFBRSxDQUFDLDZCQUFJLElBQUksQ0FBSyxDQUFDO0FBRTFFLE1BQU0sS0FBSyxHQUFnQyxDQUFDLEVBQUUsR0FBRyxFQUFFLEVBQUUsSUFBSSxFQUFFLEdBQUcsRUFBRSxHQUFHLEVBQUUsRUFBRSxFQUFFLEVBQUUsQ0FBQyxDQUMxRTtJQUNHLElBQUksS0FBSyxNQUFNLENBQUMsQ0FBQyxDQUFDLENBQ2pCO1FBQ0UseUJBQUcsSUFBSSxFQUFFLG1DQUFtQyxHQUFHLEVBQUUsa0JBQWlCOztRQUFJLEdBQUcsQ0FDcEUsQ0FDUixDQUFDLENBQUMsQ0FBQyxJQUFJO0lBQ1IseUJBQUcsSUFBSSxFQUFFLDJDQUEyQyxHQUFHLFFBQVEsR0FBRyxFQUFFLG9CQUVoRSxDQUNBLENBQ1AsQ0FBQztBQUVGLE1BQU0sTUFBTSxHQUFnQyxDQUFDLEVBQUUsR0FBRyxFQUFFLEVBQUUsTUFBTSxFQUFFLEVBQUUsRUFBRSxFQUFFLENBQUMsQ0FDbkUsK0JBQU0sTUFBTSxDQUFDLE1BQU0sQ0FBQyxTQUFTLENBQUMsTUFBTSxDQUFDLElBQUksQ0FBQyxNQUFNLENBQUMsQ0FBTyxDQUN6RCxDQUFDIiwic291cmNlc0NvbnRlbnQiOlsiaW1wb3J0ICogYXMgbW9kZWxzIGZyb20gXCIuL2RhdGEvbW9kZWxzLmpzXCI7XG5cbmV4cG9ydCBjb25zdCBIb21lOiBKU1guRkM8eyBza3VzOiBBcnJheTxtb2RlbHMuU2t1PiB9PiA9ICh7IHNrdXMgfSkgPT4gKFxuICA8bWFpbj5cbiAgICA8aDE+c3RhZGlhLm9ic2VydmVyPC9oMT5cblxuICAgIDxociAvPlxuXG4gICAgPFNrdUxpc3Q+XG4gICAgICB7c2t1cy5tYXAoc2t1ID0+IChcbiAgICAgICAgPFNrdSB7Li4uc2t1fSAvPlxuICAgICAgKSl9XG4gICAgPC9Ta3VMaXN0PlxuICA8L21haW4+XG4pO1xuXG5Ib21lLnN0eWxlID0ge1xuICBmb250U2l6ZTogXCIxNHB4XCIsXG4gIG1heFdpZHRoOiBcIjgwMHB4XCIsXG4gIG1hcmdpbjogXCI4cHggMTZweFwiLFxuICBiYWNrZ3JvdW5kSW1hZ2U6IFwidXJsKC9pbGx1ZmluY2gtdmlvbGV0c2t5LWVkaXRlZEAyeC5wbmcpXCIsXG4gIGJhY2tncm91bmRQb3NpdGlvbjogXCJ0b3AgMTZweCByaWdodCAxNnB4XCIsXG4gIGJhY2tncm91bmRSZXBlYXQ6IFwibm8tcmVwZWF0XCIsXG4gIGJhY2tncm91bmRTaXplOiBcIjY0cHhcIixcbn07XG5cbmNvbnN0IFNrdUxpc3Q6IEpTWC5GQzx7IGNoaWxkcmVuOiBKU1guUmVuZGVyYWJsZSB9PiA9ICh7IGNoaWxkcmVuIH0pID0+IChcbiAgPHNlY3Rpb24+e2NoaWxkcmVufTwvc2VjdGlvbj5cbik7XG5cbmNvbnN0IFNrdTogSlNYLkZDPG1vZGVscy5Ta3U+ID0gc2t1ID0+IChcbiAgPHNlY3Rpb24+XG4gICAgPFRpdGxlIG5hbWU9e3NrdS5uYW1lfSAvPlxuICAgIDxEZXNjcmlwdGlvbiBib2R5PXtza3UuZGVzY3JpcHRpb259IC8+XG4gICAgPExpbmtzIHNrdT17c2t1fSAvPlxuICAgIDxQcmljZXMgc2t1PXtza3V9IC8+XG4gIDwvc2VjdGlvbj5cbik7XG5cblNrdS5zdHlsZSA9IHtcbiAgZGlzcGxheTogXCJncmlkXCIsXG4gIGdyaWRUZW1wbGF0ZUNvbHVtbnM6IGBhdXRvIDIwMHB4IDEwMHB4YCxcbiAgZ3JpZFRlbXBsYXRlQXJlYXM6IGBcbiAgICBcIlRpdGxlICAgICAgICBMaW5rcyAgICAgICAgUHJpY2VzICAgICAgIFwiXG4gICAgXCJEZXNjcmlwdGlvbiAgRGVzY3JpcHRpb24gIERlc2NyaXB0aW9uICBcIlxuICBgLFxufTtcblxuY29uc3QgVGl0bGU6IEpTWC5GQzx7IG5hbWU6IHN0cmluZyB9PiA9ICh7IG5hbWUgfSkgPT4gPGgyPntuYW1lfTwvaDI+O1xuXG5jb25zdCBEZXNjcmlwdGlvbjogSlNYLkZDPHsgYm9keTogc3RyaW5nIH0+ID0gKHsgYm9keSB9KSA9PiA8cD57Ym9keX08L3A+O1xuXG5jb25zdCBMaW5rczogSlNYLkZDPHsgc2t1OiBtb2RlbHMuU2t1IH0+ID0gKHsgc2t1OiB7IHR5cGUsIGFwcCwgc2t1IH0gfSkgPT4gKFxuICA8ZGl2PlxuICAgIHt0eXBlID09PSBcImdhbWVcIiA/IChcbiAgICAgIDxzcGFuPlxuICAgICAgICA8YSBocmVmPXtgaHR0cDovL3N0YWRpYS5nb29nbGUuY29tL3BsYXllci8ke2FwcH1gfT5sYXVuY2ggZ2FtZTwvYT4gb3J7XCIgXCJ9XG4gICAgICA8L3NwYW4+XG4gICAgKSA6IG51bGx9XG4gICAgPGEgaHJlZj17YGh0dHBzOi8vc3RhZGlhLmdvb2dsZS5jb20vc3RvcmUvZGV0YWlscy8ke2FwcH0vc2t1LyR7c2t1fWB9PlxuICAgICAgdmlldyBpbiBzdG9yZVxuICAgIDwvYT5cbiAgPC9kaXY+XG4pO1xuXG5jb25zdCBQcmljZXM6IEpTWC5GQzx7IHNrdTogbW9kZWxzLlNrdSB9PiA9ICh7IHNrdTogeyBwcmljZXMgfSB9KSA9PiAoXG4gIDxkaXY+e21vZGVscy5QcmljZXMucHJvdG90eXBlLnJlbmRlci5jYWxsKHByaWNlcyl9PC9kaXY+XG4pO1xuIl19
