(async () => {})()
  .then(async () => {
    console.clear();
    const spider = new Spider();
    console.debug(spider);
    await spider.main();
    return spider;
  })
  .catch((error) => console.error(error));

const applicationId = "a4c5eb3f4e614b7fadbba64cba68f849rcp1";
const gameSku = "71e7c4fc5ca24f0f8301da6d042bf18e";

type Proto = Array<Proto | number | string | boolean | null> | Array<any>;

class Spider {
  public async main() {
    await this.loadSeedPages();
    await this.spiderAll();
    this.download();
  }

  public constructor(
    private products: Record<Product["sku"], Product> = {},
    private knownSkus: Record<Product["sku"], true> = {}, 
    private spideredSkus: Record<Product["sku"], true> = {},
  ) {}

  /// Loads data from some initial pages from which we should be able to find all SKUs.
  private async loadSeedPages() {
    await this.loadHome();
    await this.loadStoreList(3);
    await this.loadStoreList(45);
    await this.loadStoreProduct(applicationId, gameSku);
  }

  private async spiderAll() {
    while (this.knownSkus.size < this.spideredSkus.size) {
      let next: string;
      for (const candidate of Object.keys(this.knownSkus)) {
        if (!this.spideredSkus.hasOwnProperty(candidate)) {
          next = candidate;
          break;
        }
      }

      // I can't spider without also knowing the app ID?
      // Which I should?
      await this.loadProductPage();
      this.spideredSkus[next] = true;
    }

  }

  /// Downloads a JSON file with all loaded data.
  public download() {
    const json = JSON.stringify({
      products: this.products,
    }, null, 2);
    const href = URL.createObjectURL(
      new Blob([json], {
        type: "application/octet-stream",
      })
    );
    const el = Object.assign(document.createElement("a"), {
      download: "data.json",
      href,
    });
    document.body.appendChild(el);
    el.click();
    document.body.removeChild(el);
  }

  private loadProduct(data: Proto): Product {
    const productTypeId = data[6];
    const product =
      productTypeId === 1
        ? new Game(data[4], data[0], "game", data[1], data[5], data[9])
        : productTypeId === 2
        ? new AddOn(data[4], data[0], "addon", data[1])
        : productTypeId === 3
        ? new Bundle(
            data[4],
            data[0],
            "bundle",
            data[1],
            data[14][0].map((x) => x[0])
          )
        : (new BaseProduct(
            data[4],
            data[0],
            productTypeId,
            data[1]
          ) as Product);
    const previous = this.products[product.sku];
    return (this.products[product.sku] = previous
      ? Object.assign(previous, product)
      : product);
  }

  private async loadHome() {
    const page = await fetchPreloadData("/home");
  }

  private async loadStoreList(number: number) {
    const page = await fetchPreloadData(`/store/list/${number}`);
  }

  private async loadStoreProduct(applicationId: string, sku: string) {
    const page = await fetchPreloadData(`
    "/store/details/${applicationId}/sku/${sku}"`);
  }

  private async loadPlayerProfile(playerId: string) {
    const page = await fetchPreloadData(
      `/profile/${playerId}/gameactivities/all`
    );
  }

  private async loadAchivements(applicationId: string, playerId?: string) {
    const page = await fetchPreloadData(
      playerId
        ? `/profile/${playerId}/detail/${applicationId}`
        : `/profile/detail/${applicationId}`
    );
  }
}

class BaseProduct {
  constructor(
    readonly appId = "a4c5eb3f4e614b7fadbba64cba68f849rcp1",
    readonly sku = "71e7c4fc5ca24f0f8301da6d042bf18e",
    readonly productType: "game" | "addon" | "bundle" = "game",
    readonly name = "PLAYERUNKNOWN'S BATTLEGROUNDS"
  ) {}
}

class Game extends BaseProduct {
  constructor(
    readonly appId = "a4c5eb3f4e614b7fadbba64cba68f849rcp1",
    readonly sku = "71e7c4fc5ca24f0f8301da6d042bf18e",
    readonly productType = "game" as const,
    readonly name = "PLAYERUNKNOWN'S BATTLEGROUNDS",
    readonly somename = "PUBGBATTLEGROUNDS01",
    readonly description = ""
  ) {
    super(appId, sku, productType, name);
  }
}

class GameAchievements {
  constructor(readonly appId = "a4c5eb3f4e614b7fadbba64cba68f849rcp1") {}
}

class Achievements {}

class Bundle extends BaseProduct {
  constructor(
    readonly appId = "a4c5eb3f4e614b7fadbba64cba68f849rcp1",
    readonly sku = "71e7c4fc5ca24f0f8301da6d042bf18e",
    readonly productType = "bundle" as const,
    readonly name = "PLAYERUNKNOWN'S BATTLEGROUNDS",
    readonly bundleSkus = new Array<string>()
  ) {
    super(appId, sku, productType, name);
  }
}

class AddOn extends BaseProduct {
  constructor(
    readonly appId = "a4c5eb3f4e614b7fadbba64cba68f849rcp1",
    readonly sku = "71e7c4fc5ca24f0f8301da6d042bf18e",
    readonly productType = "addon" as const,
    readonly name = "PLAYERUNKNOWN'S BATTLEGROUNDS"
  ) {
    super(appId, sku, productType, name);
  }
}

type Product = Game | Bundle | AddOn;

const fetchPreloadData = async (
  url: string = "https://stadia.google.com/"
): Promise<Record<string, Array<any>>> => {
  // TODO: index by AF_dataServiceRequests
  await new Promise((resolve) =>
    setTimeout(resolve, Math.random() * 3500 + 1500)
  );
  const response = await fetch(url);
  const html = await response.text();
  const document = new DOMParser().parseFromString(html, "text/html");
  const scripts = Array.from(document.scripts);
  const contents = scripts.map((s) => s.textContent.trim()).filter(Boolean);

  const dataServiceRequestsPattern = /^var *AF_initDataKeys[^]*?var *AF_dataServiceRequests *= *({[^]*}); *?var /;

  const dataServiceRequests = contents
    .map((s) => s.match(dataServiceRequestsPattern))
    .filter(Boolean)
    .map((matches) =>
      JSON.parse(
        matches[1]
          .replace(/{id:/g, '{"id":')
          .replace(/,request:/g, ',"request":')
          .replace(/'/g, '"')
      )
    )
    .map((requests) =>
      Object.fromEntries(
        Object.entries(requests).map(([key, value]: any) => [
          key,
          value.id + JSON.stringify(value.request),
        ])
      )
    )[0];

  const dataCallbackPattern = /^ *AF_initDataCallback *\( *{ *key *: *'([^]*?)' *,[^]*?data: *function *\( *\){ *return *([^]*)\s*}\s*}\s*\)\s*;?\s*$/;
  const dataServiceLoads = contents
    .map((s) => s.match(dataCallbackPattern))
    .filter(Boolean)
    .map((matches) => JSON.parse(matches[2]));

  const preloadedData = Object.fromEntries(
    Object.entries(dataServiceLoads)
      .map(([key, value]) => [
        ["_" + key, value],
        [dataServiceRequests["ds:" + key], value],
      ])
      .flat()
      .map(([key, value]) => [
        [key, value],
        [key.split("[")[0], value],
      ])
      .flat()
      .map(([key, value]) => [
        [key, value],
        [requestLabels[key] || key, value],
      ])
      .flat()
  );

  console.debug(url, preloadedData);

  return preloadedData;
};
