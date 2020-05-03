declare type ProtoData = any &
  Array<ProtoData | number | string | boolean | null>;
declare type Sku = Game | AddOn | Bundle | Subscription;
declare class Spider {
  private readonly skus;
  private readonly spidered;
  private readonly start;
  private readonly done;
  constructor(
    skus?: Record<Sku["sku"], Sku>,
    spidered?: Record<Sku["sku"], true>
  );
  load(): Promise<Record<string, Sku>>;
  private spider;
  download(): void;
  private loadSkuList;
  private loadSkuDetails;
  private loadSkuData;
  private loadSku;
  fetchPreloadData(url: string): Promise<any>;
}
declare class CommonSku {
  readonly appId: string;
  readonly sku: string;
  readonly skuType: "game" | "addon" | "bundle" | "subscription";
  readonly name: string;
  readonly somename: string;
  readonly description: string;
  constructor(
    appId: string,
    sku: string,
    skuType: "game" | "addon" | "bundle" | "subscription",
    name: string,
    somename: string,
    description: string
  );
}
declare class Game extends CommonSku {
  readonly skuType: "game";
}
declare class Bundle extends CommonSku {
  readonly skus: Array<string>;
  constructor(
    appId: string,
    sku: string,
    skuType: "bundle",
    name: string,
    somename: string,
    description: string,
    skus: Array<string>
  );
}
declare class AddOn extends CommonSku {
  readonly skuType: "game";
}
declare class Subscription extends CommonSku {
  readonly skus: Array<string>;
  constructor(
    appId: string,
    sku: string,
    skuType: "subscription",
    name: string,
    somename: string,
    description: string,
    skus: Array<string>
  );
}
