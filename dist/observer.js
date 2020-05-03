"use strict";
(async () => { })()
    .then(async () => {
    console.clear();
    const spider = new Spider();
    console.debug(spider);
    await spider.main();
    return spider;
})
    .catch((error) => console.error(error));
const playerId = "956082794034380385";
const applicationId = "a4c5eb3f4e614b7fadbba64cba68f849rcp1";
const gameSku = "71e7c4fc5ca24f0f8301da6d042bf18e";
class Spider {
    constructor(players = {}, products = {}, spideredSkus = {}) {
        this.players = players;
        this.products = products;
        this.spideredSkus = spideredSkus;
    }
    async main() {
        await this.loadHome();
        await this.loadStoreList(3);
        await this.loadStoreList(45);
        await this.loadPlayerProfile(playerId);
        await this.loadPlayerGame(playerId, gameSku);
        await this.loadStoreProduct(applicationId, gameSku);
        this.download();
    }
    loadPlayer(data) {
        const player = new Player(data[5], data[0][0], data[0][1], data[3], data[1][0], data[1][1]);
        const previous = this.players[player.playerId];
        return (this.players[player.playerId] = previous
            ? Object.assign(previous, player)
            : player);
    }
    loadProduct(data) {
        const productTypeId = data[6];
        const product = productTypeId === 1
            ? new Game(data[4], data[0], "game", data[1], data[5], data[9])
            : productTypeId === 2
                ? new AddOn(data[4], data[0], "addon", data[1])
                : productTypeId === 3
                    ? new Bundle(data[4], data[0], "bundle", data[1], data[14][0].map((x) => x[0]))
                    : new BaseProduct(data[4], data[0], productTypeId, data[1]);
        const previous = this.products[product.sku];
        return (this.products[product.sku] = previous
            ? Object.assign(previous, product)
            : product);
    }
    async loadHome() {
        const page = await fetchPreloadData("/home");
    }
    async loadStoreList(number) {
        const page = await fetchPreloadData(`/store/list/${number}`);
    }
    async loadStoreProduct(applicationId, sku) {
        const page = await fetchPreloadData(`
    "/store/details/${applicationId}/sku/${sku}"`);
    }
    async loadPlayerProfile(playerId) {
        const page = await fetchPreloadData(`/profile/${playerId}/gameactivities/all`);
    }
    async loadAchivements(applicationId, playerId) {
        const page = await fetchPreloadData(playerId
            ? `/profile/${playerId}/detail/${applicationId}`
            : `/profile/detail/${applicationId}`);
    }
    download() {
        const json = JSON.stringify(this, null, 2);
        const href = URL.createObjectURL(new Blob([json], {
            type: "application/octet-stream",
        }));
        const el = Object.assign(document.createElement("a"), {
            download: "data.json",
            href,
        });
        document.body.appendChild(el);
        el.click();
        document.body.removeChild(el);
    }
}
const requestLabels = {
    "PtnQCd[]": "activeSubscriptions",
    "Qc7K6[]": "ownedGames",
    "LV6ate[]": "currentPlayer",
    "CmnEcf[[3]]": "alsoCurrentPlayer",
    WwD3rb: "productList",
    "WwD3rb[3]": "allGames",
    "WwD3rb[45]": "stadiaProDeals",
    "T9Kmu[]": "recentCaptures",
    Q6jt8c: "thisPlayer",
    GRn9Gb: "myGames",
    FLCvtc: "bundles",
    Qc7K6: "addons",
    ZAm7We: "product",
    D0Amud: "main",
    e7h9qd: "friendsWhoPlay",
};
class Player {
    constructor(playerId = "956082794034380385", gamertagName = "Jeremy", gamertagNumber = "0000", somename = "JEREMY", avatarId = "s00056", avatarUrl = "https://www.gstatic.com/stadia/gamers/avatars/mdpi/avatar_56.png") {
        this.playerId = playerId;
        this.gamertagName = gamertagName;
        this.gamertagNumber = gamertagNumber;
        this.somename = somename;
        this.avatarId = avatarId;
        this.avatarUrl = avatarUrl;
    }
}
class BaseProduct {
    constructor(appId = "a4c5eb3f4e614b7fadbba64cba68f849rcp1", sku = "71e7c4fc5ca24f0f8301da6d042bf18e", productType = "game", name = "PLAYERUNKNOWN'S BATTLEGROUNDS") {
        this.appId = appId;
        this.sku = sku;
        this.productType = productType;
        this.name = name;
    }
}
class Game extends BaseProduct {
    constructor(appId = "a4c5eb3f4e614b7fadbba64cba68f849rcp1", sku = "71e7c4fc5ca24f0f8301da6d042bf18e", productType = "game", name = "PLAYERUNKNOWN'S BATTLEGROUNDS", somename = "PUBGBATTLEGROUNDS01", description = "") {
        super(appId, sku, productType, name);
        this.appId = appId;
        this.sku = sku;
        this.productType = productType;
        this.name = name;
        this.somename = somename;
        this.description = description;
    }
}
class GameAchievements {
    constructor(appId = "a4c5eb3f4e614b7fadbba64cba68f849rcp1") {
        this.appId = appId;
    }
}
class Achievements {
}
class Bundle extends BaseProduct {
    constructor(appId = "a4c5eb3f4e614b7fadbba64cba68f849rcp1", sku = "71e7c4fc5ca24f0f8301da6d042bf18e", productType = "bundle", name = "PLAYERUNKNOWN'S BATTLEGROUNDS", bundleSkus = new Array()) {
        super(appId, sku, productType, name);
        this.appId = appId;
        this.sku = sku;
        this.productType = productType;
        this.name = name;
        this.bundleSkus = bundleSkus;
    }
}
class AddOn extends BaseProduct {
    constructor(appId = "a4c5eb3f4e614b7fadbba64cba68f849rcp1", sku = "71e7c4fc5ca24f0f8301da6d042bf18e", productType = "addon", name = "PLAYERUNKNOWN'S BATTLEGROUNDS") {
        super(appId, sku, productType, name);
        this.appId = appId;
        this.sku = sku;
        this.productType = productType;
        this.name = name;
    }
}
const fetchPreloadData = async (url = "https://stadia.google.com/") => {
    // TODO: index by AF_dataServiceRequests
    await new Promise((resolve) => setTimeout(resolve, Math.random() * 3500 + 1500));
    const response = await fetch(url);
    const html = await response.text();
    const document = new DOMParser().parseFromString(html, "text/html");
    const scripts = Array.from(document.scripts);
    const contents = scripts.map((s) => s.textContent.trim()).filter(Boolean);
    const dataServiceRequestsPattern = /^var *AF_initDataKeys[^]*?var *AF_dataServiceRequests *= *({[^]*}); *?var /;
    const dataServiceRequests = contents
        .map((s) => s.match(dataServiceRequestsPattern))
        .filter(Boolean)
        .map((matches) => JSON.parse(matches[1]
        .replace(/{id:/g, '{"id":')
        .replace(/,request:/g, ',"request":')
        .replace(/'/g, '"')))
        .map((requests) => Object.fromEntries(Object.entries(requests).map(([key, value]) => [
        key,
        value.id + JSON.stringify(value.request),
    ])))[0];
    const dataCallbackPattern = /^ *AF_initDataCallback *\( *{ *key *: *'([^]*?)' *,[^]*?data: *function *\( *\){ *return *([^]*)\s*}\s*}\s*\)\s*;?\s*$/;
    const dataServiceLoads = contents
        .map((s) => s.match(dataCallbackPattern))
        .filter(Boolean)
        .map((matches) => JSON.parse(matches[2]));
    const preloadedData = Object.fromEntries(Object.entries(dataServiceLoads)
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
        .flat());
    console.debug(url, preloadedData);
    return preloadedData;
};
//# sourceMappingURL=observer.js.map