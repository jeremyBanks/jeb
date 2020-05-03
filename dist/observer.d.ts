declare const playerId = "956082794034380385";
declare const applicationId = "a4c5eb3f4e614b7fadbba64cba68f849rcp1";
declare const gameSku = "71e7c4fc5ca24f0f8301da6d042bf18e";
declare type Proto = Array<Proto | number | string | boolean | null> | Array<any>;
declare class Spider {
    private players;
    private products;
    private spideredSkus;
    main(): Promise<void>;
    constructor(players?: Record<Player["playerId"], Player>, products?: Record<Product["sku"], Product>, spideredSkus?: Record<Product["sku"], true>);
    private loadPlayer;
    private loadProduct;
    private loadHome;
    private loadStoreList;
    private loadStoreProduct;
    private loadPlayerProfile;
    private loadAchivements;
    download(): void;
}
declare const requestLabels: {
    "PtnQCd[]": string;
    "Qc7K6[]": string;
    "LV6ate[]": string;
    "CmnEcf[[3]]": string;
    WwD3rb: string;
    "WwD3rb[3]": string;
    "WwD3rb[45]": string;
    "T9Kmu[]": string;
    Q6jt8c: string;
    GRn9Gb: string;
    FLCvtc: string;
    Qc7K6: string;
    ZAm7We: string;
    D0Amud: string;
    e7h9qd: string;
};
declare class Player {
    readonly playerId: string;
    readonly gamertagName: string;
    readonly gamertagNumber: string;
    readonly somename: string;
    readonly avatarId: string;
    readonly avatarUrl: string;
    constructor(playerId?: string, gamertagName?: string, gamertagNumber?: string, somename?: string, avatarId?: string, avatarUrl?: string);
}
declare class BaseProduct {
    readonly appId: string;
    readonly sku: string;
    readonly productType: "game" | "addon" | "bundle";
    readonly name: string;
    constructor(appId?: string, sku?: string, productType?: "game" | "addon" | "bundle", name?: string);
}
declare class Game extends BaseProduct {
    readonly appId: string;
    readonly sku: string;
    readonly productType: "game";
    readonly name: string;
    readonly somename: string;
    readonly description: string;
    constructor(appId?: string, sku?: string, productType?: "game", name?: string, somename?: string, description?: string);
}
declare class GameAchievements {
    readonly appId: string;
    constructor(appId?: string);
}
declare class Achievements {
}
declare class Bundle extends BaseProduct {
    readonly appId: string;
    readonly sku: string;
    readonly productType: "bundle";
    readonly name: string;
    readonly bundleSkus: string[];
    constructor(appId?: string, sku?: string, productType?: "bundle", name?: string, bundleSkus?: string[]);
}
declare class AddOn extends BaseProduct {
    readonly appId: string;
    readonly sku: string;
    readonly productType: "addon";
    readonly name: string;
    constructor(appId?: string, sku?: string, productType?: "addon", name?: string);
}
declare type Product = Game | Bundle | AddOn;
declare const fetchPreloadData: (url?: string) => Promise<Record<string, any[]>>;
