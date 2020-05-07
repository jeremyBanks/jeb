import * as records from "./records.js";

import { AddOn, Bundle, DataStore, Game, Sku, Subscription } from "./data.js";

export const spider = async () => {
  const storage = await DataStore.open();
  const spider = new Spider();
  console.debug("starting spider", spider);
  const data = await spider.load();
  console.debug("completed spider", spider, data);
  await storage.save();
  spider.download();
};

type ProtoData = any & Array<ProtoData | number | string | boolean | null>;

class Spider {
  private readonly start: () => void;
  private readonly done: Promise<unknown>;
  public constructor(
    private readonly skus: Record<Sku["sku"], Sku> = {},
    private readonly spidered: Record<Sku["sku"], true> = {}
  ) {
    let start: () => void;
    const started = new Promise((resolve) => (start = resolve));
    this.start = start;
    this.done = started
      .then(() => this.spider())
      .then(() => {
        Object.freeze(this.skus);
        for (const sku of Object.values(this.skus)) {
          Object.freeze(sku);
        }
      });
  }

  public async load() {
    this.start();
    await this.done;
    return this.output();
  }

  private async spider() {
    {
      await this.loadSkuDetails("59c8314ac82a456ba61d08988b15b550");
      await this.loadSkuDetails("b171fc78d4e1496d9536d585257a771e");
      await this.loadSkuDetails("4950959380034dcda0aecf98f675e11f");
      await this.loadSkuDetails("2e07db5d338d40cb9eac9deae4154f11");
      await this.loadSkuList(3);
      await this.loadSkuList(2001);
      await this.loadSkuList(45);
      await this.loadSkuList(6);
      throw "TODO: remove me";
    }

    await this.loadSkuList(3);

    while (Object.keys(this.skus).length > Object.keys(this.spidered).length) {
      for (const sku of Object.values(this.skus)) {
        if (this.spidered[sku.sku] !== true) {
          console.debug("spidering ", sku);
          await this.loadSkuDetails(sku.sku);
          this.spidered[sku.sku] = true;
        }
      }
    }
  }

  public output() {
    return records.sorted(
      Object.fromEntries(
        Object.values(this.skus)
          .map((sku) => records.sorted(sku))
          .map((sku) => [sku.localKey, sku])
      )
    );
  }

  public download() {
    const output = this.output();
    const json = JSON.stringify(output, null, 2);
    // XXX: this blob is leaked
    const href = URL.createObjectURL(
      new Blob([json], {
        type: "application/json",
      })
    );
    const el = Object.assign(document.createElement("a"), {
      download: "skus.json",
      href,
    });
    document.body.appendChild(el);
    el.click();
    document.body.removeChild(el);
  }

  private async loadSkuList(number: number) {
    await this.fetchPreloadData(`store/list/${number}`);
  }

  private async loadSkuDetails(sku: string) {
    await this.fetchPreloadData(`store/details/_/sku/${sku}`);
  }

  private loadSkuData(data: ProtoData, pricing: ProtoData): Sku {
    const typeId = data[6];

    console.log({ pricing });
    // pricing should include normal price, pro price,
    // current sale prices for each, the start and end dates of each sale

    let sku;
    if (typeId === 1) {
      sku = new Game(data[4], data[0], "game", data[1], data[5], data[9]);
    } else if (typeId === 2) {
      sku = new AddOn(data[4], data[0], "addon", data[1], data[5], data[9]);
    } else if (typeId === 3) {
      sku = new Bundle(
        data[4],
        data[0],
        "bundle",
        data[1],
        data[5],
        data[9],
        data[14][0].map((x: any) => x[0])
      );
    } else if (typeId === 5) {
      sku = new Subscription(
        data[4],
        data[0],
        "subscription",
        data[1],
        data[5],
        data[9],
        data[14][0].map((x: any) => x[0])
      );
    } else {
      throw new Error(
        `unexpected sku type id ${JSON.stringify(typeId, null, 4)}`
      );
    }

    return this.loadSku(sku);
  }

  private loadSku(sku: Sku): Sku {
    const existing = this.skus[sku.sku];
    if (existing) {
      const existingJson = JSON.stringify(existing);
      const newJson = JSON.stringify(sku);
      if (existingJson !== newJson) {
        console.warn(`skus had same ids but different properties.`);
        Object.assign(existing, sku);
      }
      return existing;
    } else {
      this.skus[sku.sku] = sku;
      return sku;
    }
  }

  async fetchPreloadData(path: string) {
    await new Promise((resolve) =>
      setTimeout(resolve, Math.random() * 1_000 + 2_000)
    );
    const response = await fetch("https://stadia.google.com/" + path, {
      headers: {
        "User-Agent":
          "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/81.0.4044.138 Safari/537.36",
      },
    });
    const html = await response.text();
    const document = new DOMParser().parseFromString(html, "text/html");
    const scripts = Array.from(document.scripts);
    const contents = scripts.map((s) => s.textContent?.trim()).filter(Boolean);

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
          Object.entries(requests).map(([key, value]: any) => [key, value])
        )
      )[0];

    const dataCallbackPattern = /^ *AF_initDataCallback *\( *{ *key *: *'ds:([0-9]+?)' *,[^]*?data: *function *\( *\){ *return *([^]*)\s*}\s*}\s*\)\s*;?\s*$/;
    const dataServiceLoads: Array<any> = [];
    for (const matches of contents
      .map((s) => s.match(dataCallbackPattern))
      .filter(Boolean)) {
      dataServiceLoads[matches[1]] = JSON.parse(matches[2]);
    }
    6;
    const dataServiceRpcPrefixes = Object.values(dataServiceRequests).map(
      (x: any) => {
        const pieces = [x.id, ...x.request];
        const aliases = [];
        aliases.push(
          `${pieces[0]}_${pieces.filter((x: any) => x != null).length - 1}s${
            pieces.filter(Boolean).length - 1
          }t${pieces.length - 1}a`
        );
        while (pieces.length) {
          aliases.push(pieces.join("_"));
          pieces.pop();
        }
        return aliases;
      }
    );

    const preload = Object.values(dataServiceLoads);
    const rpc: any = {};
    const loaded = {};

    const loaders: any = {
      WwD3rb: (data: ProtoData) => {
        const skus = data[2].map((p: any) => this.loadSkuData(p[9]));
        return { skus };
      },

      FWhQV_24r: (data: ProtoData) => {
        const gameData = data[18]?.[0]?.[9];
        const gamePricingData = data[18]?.[0]?.[15]?.[0];
        const game = gameData && this.loadSkuData(gameData, gamePricingData);

        const addons = (data[19] as any)?.map((x: any) =>
          this.loadSkuData(x[9])
        );

        const skuData = data[16];
        const skuPricingData = data[21]?.[0];
        const sku = skuData && this.loadSkuData(skuData, skuPricingData);

        return { sku, game, addons };
      },

      SYcsTd: (data: ProtoData) => {
        const subscriptionDatas = data[2]?.map((x: any) => x[9]) ?? [];
        const subscriptions = subscriptionDatas.map((s) => this.loadSkuData(s));
        if (subscriptions?.length) return { subscriptions };
        else return {};
      },

      ZAm7W: (data: ProtoData) => {
        const bundles = data[1].map((x: any) => this.loadSkuData(x[9]));
        return { bundles };
      },
    };

    for (const [i, data] of Object.entries(preload)) {
      for (const prefix of dataServiceRpcPrefixes[i]) {
        for (const suffix of ["", "_" + Object.keys(data).length + "r"]) {
          rpc[prefix + suffix] = data;
          const loader = loaders[prefix + suffix];
          if (loader) {
            Object.assign(loaded, loader(data));
          }
        }
      }
    }

    const data = Object.assign(
      Object.create({
        preload,
        rpc,
      }),
      loaded
    );

    console.debug(path, data);

    return data;
  }
}
