import { storage } from "./storage.js";
import * as records from "./records.js";

export const spider = async () => {
  const usage = await storage.usage();
  const schema = 20200202;
  if (0 !== (await storage.get("schema"))) {
    console.debug(`Resetting storage for schema ${schema}.`);
    await storage.clear();
    await storage.set("schema", 0);
  }
  console.debug(`Using ${usage} bytes of storage with schema ${schema}.`);

  await new Promise((resolve) => setTimeout(resolve, 10));
  try {
    const spider = new Spider();
    window["spider"] = spider;
    console.debug("starting spider", spider);
    const data = await spider.load();
    console.debug("completed spider", spider, data);
    spider.download();
    await storage.set("skus", spider.skus);
  } catch (error) {
    console.error(error);
  }
};

type ProtoData = any & Array<ProtoData | number | string | boolean | null>;
type Sku = Game | AddOn | Bundle | Subscription;

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
    return this.skus;
  }

  private async spider() {
    await this.loadSkuDetails("59c8314ac82a456ba61d08988b15b550");
    await this.loadSkuDetails("b171fc78d4e1496d9536d585257a771e");
    await this.loadSkuDetails("4950959380034dcda0aecf98f675e11f");
    await this.loadSkuDetails("2e07db5d338d40cb9eac9deae4154f11");
    await this.loadSkuList(3);
    await this.loadSkuList(2001);
    await this.loadSkuList(45);
    await this.loadSkuList(6);

    console.warn(
      "TODO: re-enable spidering once we've finished the above tweaks."
    );
    return;
    while (Object.keys(this.skus).length > Object.keys(this.spidered).length) {
      for (const sku of Object.values(this.skus)) {
        if (this.spidered[sku.sku] !== true) {
          console.debug("spidering ", sortySlugy(sku), sku);
          await this.loadSkuDetails(sku.sku);
          this.spidered[sku.sku] = true;
        }
      }
    }
  }

  public download() {
    const output = records.sorted(
      Object.fromEntries(
        Object.values(this.skus)
          .map((sku) => records.sorted(sku))
          .map((sku) => [sku.localKey, sku])
      )
    );
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
    const response = await fetch("https://stadia.google.com/" + path);
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

const sortySlugy = (sku: Sku | CommonSku) => {
  const length = 32;
  const maxNameLength = 23;
  const typeTag =
    ({ game: "g", addon: "o", bundle: "x", subscription: "c" } as any)[
      sku.type
    ] ?? `?${sku.type}?`;
  const idsPrefix = sku.app.slice(0, 6) + sku.sku.slice(0, 2);
  const idsRest = sku.app.slice(6) + sku.sku.slice(2);

  let name = (sku.name + sku.internalSlug)
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "");

  if (name.length > maxNameLength) {
    const letterCounts: Record<string, number> = {};
    for (const letter of name) {
      letterCounts[letter] = (letterCounts[letter] || 0) + 1;
    }
    while (name.length > maxNameLength) {
      const mostFrequentCount = Math.max(...Object.values(letterCounts));
      const mostFrequent = Object.entries(letterCounts)
        .filter(([_letter, count]) => count == mostFrequentCount)
        .map(([letter, _count]) => letter);
      for (let i = name.length - 1; i >= 0; i -= 1) {
        const letter = name[i];
        if (mostFrequent.includes(letter)) {
          name = name.slice(0, i) + name.slice(i + 1);
          letterCounts[letter] -= 1;
          break;
        }
      }
    }
  }

  return (typeTag + idsPrefix + name + idsRest).slice(0, length);
};

class CommonSku {
  constructor(
    readonly app: string,
    readonly sku: string,
    readonly type: "game" | "addon" | "bundle" | "subscription",
    readonly name: string,
    readonly internalSlug: string,
    readonly description: string
  ) {
    this.localKey = sortySlugy(this);
  }
  readonly localKey: string;
}

class Game extends CommonSku {
  constructor(
    app: string,
    sku: string,
    readonly type = "game" as const,
    name: string,
    readonly internalSlug: string,
    readonly description: string
  ) {
    super(app, sku, type, name, internalSlug, description);
  }
}

class AddOn extends CommonSku {
  constructor(
    app: string,
    sku: string,
    readonly type = "addon" as const,
    name: string,
    readonly internalSlug: string,
    readonly description: string
  ) {
    super(app, sku, type, name, internalSlug, description);
  }
}

class Bundle extends CommonSku {
  constructor(
    app: string,
    sku: string,
    readonly type = "bundle" as const,
    name: string,
    readonly internalSlug: string,
    readonly description: string,
    readonly skus: Array<string>
  ) {
    super(app, sku, type, name, internalSlug, description);
  }
}

class Subscription extends CommonSku {
  constructor(
    app: string,
    sku: string,
    readonly type = "subscription" as const,
    name: string,
    readonly internalSlug: string,
    readonly description: string,
    readonly skus: Array<string>
  ) {
    super(app, sku, type, name, internalSlug, description);
  }
}
