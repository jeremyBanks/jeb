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
  readonly type: "game" | "addon" | "bundle" | "subscription";
  readonly name: string;
  readonly handle: string;
  readonly description: string;
  constructor(
    appId: string,
    sku: string,
    type: "game" | "addon" | "bundle" | "subscription",
    name: string,
    handle: string,
    description: string
  );
}
declare class Game extends CommonSku {
  readonly type: "game";
}
declare class AddOn extends CommonSku {
  readonly type: "addon";
}
declare class Bundle extends CommonSku {
  readonly skus: Array<string>;
  constructor(
    appId: string,
    sku: string,
    type: "bundle",
    name: string,
    handle: string,
    description: string,
    skus: Array<string>
  );
}
declare class Subscription extends CommonSku {
  readonly skus: Array<string>;
  constructor(
    appId: string,
    sku: string,
    type: "subscription",
    name: string,
    handle: string,
    description: string,
    skus: Array<string>
  );
}
