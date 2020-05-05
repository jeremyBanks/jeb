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
  fetchPreloadData(path: string): Promise<any>;
}
declare class CommonSku {
  readonly app: string;
  readonly sku: string;
  readonly type: "game" | "addon" | "bundle" | "subscription";
  readonly name: string;
  constructor(
    app: string,
    sku: string,
    type: "game" | "addon" | "bundle" | "subscription",
    name: string
  );
  key(): string;
}
declare class Game extends CommonSku {
  readonly type: "game";
  constructor(app: string, sku: string, type: "game", name: string);
}
declare class AddOn extends CommonSku {
  readonly type: "addon";
  constructor(app: string, sku: string, type: "addon", name: string);
}
declare class Bundle extends CommonSku {
  readonly type: "bundle";
  readonly skus: Array<string>;
  constructor(
    app: string,
    sku: string,
    type: "bundle",
    name: string,
    skus: Array<string>
  );
}
declare class Subscription extends CommonSku {
  readonly type: "subscription";
  readonly skus: Array<string>;
  constructor(
    app: string,
    sku: string,
    type: "subscription",
    name: string,
    skus: Array<string>
  );
}
