declare type ProtoData = any &
  Array<ProtoData | number | string | boolean | null>;
declare class Spider {
  private readonly products;
  private readonly spidered;
  private readonly start;
  private readonly done;
  constructor(
    products?: Record<Product["sku"], Product>,
    spidered?: Record<Product["sku"], true>
  );
  load(): Promise<Record<string, Product>>;
  private spider;
  download(): void;
  private loadProductList;
  private loadProductDetails;
  private loadProductData;
  private loadProduct;
  fetchPreloadData(url: string): Promise<any>;
}
declare class Product {
  readonly appId: string;
  readonly sku: string;
  readonly productType: "game" | "addon" | "bundle";
  readonly name: string;
  constructor(
    appId?: string,
    sku?: string,
    productType?: "game" | "addon" | "bundle",
    name?: string
  );
}
declare class Game extends Product {
  readonly appId: string;
  readonly sku: string;
  readonly productType: "game";
  readonly name: string;
  readonly somename: string;
  readonly description: string;
  constructor(
    appId?: string,
    sku?: string,
    productType?: "game",
    name?: string,
    somename?: string,
    description?: string
  );
}
declare class Bundle extends Product {
  readonly appId: string;
  readonly sku: string;
  readonly productType: "bundle";
  readonly name: string;
  readonly bundleSkus: string[];
  constructor(
    appId?: string,
    sku?: string,
    productType?: "bundle",
    name?: string,
    bundleSkus?: string[]
  );
}
declare class AddOn extends Product {
  readonly appId: string;
  readonly sku: string;
  readonly productType: "addon";
  readonly name: string;
  constructor(
    appId?: string,
    sku?: string,
    productType?: "addon",
    name?: string
  );
}
