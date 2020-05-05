"use strict";
(async () => {
    document.title = "stadia.observer";
    document.head.appendChild(Object.assign(document.createElement("link"), {
        rel: "icon",
        href: "/illufinch-violetsky-edited@4x.png",
    }));
    const renderContent = (content) => {
        if (content === null || content === undefined) {
            return document.createElement("span");
        }
        else if (content instanceof Element) {
            return content;
        }
        else if (typeof content === "string" || typeof content === "number") {
            return Object.assign(document.createElement("span"), {
                textContent: String(content),
            });
        }
        else if (content instanceof Node) {
            const container = document.createElement("span");
            container.appendChild(content);
            return container;
        }
        else {
            console.log({ content });
            const container = document.createElement("span");
            for (const child of content) {
                container.appendChild(renderContent(child));
            }
            return container;
        }
    };
    const renderTemplate = (template, slots = {}, target = document.createElement("span")) => {
        while (target.lastChild) {
            target.removeChild(target.lastChild);
        }
        if (target.shadowRoot) {
            while (target.shadowRoot.lastChild) {
                target.removeChild(target.shadowRoot.lastChild);
            }
        }
        else {
            target.attachShadow({ mode: "open" });
        }
        target.shadowRoot.appendChild(template.cloneNode(true));
        for (const [key, content] of Object.entries(slots)) {
            if (template.querySelector(`slot[name=${CSS.escape(key)}]`)) {
                let contentElement = renderContent(content);
                contentElement.slot = key;
                target.appendChild(contentElement);
            }
        }
        return target;
    };
    const templates = Object.fromEntries(Array.from(new DOMParser()
        .parseFromString(await (await fetch("/templates.html")).text(), "text/html")
        .querySelectorAll("template")).map((el) => [el.id, el.content]));
    const render = (name, props, child) => {
        if (name instanceof DocumentFragment) {
            return renderTemplate(name, { child, ...props });
        }
        else if (templates.hasOwnProperty(name)) {
            return renderTemplate(templates[name], { child, ...props });
        }
        else {
            const el = document.createElement(name);
            Object.assign(el, props);
            el.appendChild(renderContent(child));
            return el;
        }
    };
    const skus = Object.values(await (await fetch("/skus.json")).json());
    const skusById = Object.fromEntries(skus.map((sku) => [sku.sku, sku]));
    const subscriptions = skus.filter((sku) => sku.type === "subscription");
    const games = skus
        .filter((sku) => sku.type === "game")
        .map((game) => Object.assign(game, {
        skus: skus.filter((sku) => sku.app === game.app),
        addons: skus.filter((sku) => sku.app === game.app && sku.type === "addon"),
        bundles: skus.filter((sku) => sku.app === game.app && sku.type === "bundle"),
    }));
    document.body.appendChild(render("home", { title: document.title, buttons: render("button", { onclick: () => import("/spider.js") }, "\uD83D\uDD77\uFE0Fspider stadia"), games: games.map((game) => (render("game", Object.assign({}, game)))) }));
})();
//# sourceMappingURL=index.js.map