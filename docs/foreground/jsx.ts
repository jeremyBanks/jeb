export type Renderable = JSX.Renderable;
export type FC<Props> = JSX.FC<Props>;

const renderChild = (content: Renderable): Node => {
  if (content === null || content === undefined) {
    return document.createElement("span");
  } else if (content instanceof Node) {
    return content;
  } else if (
    typeof content === "string" ||
    typeof content === "number" ||
    typeof content === "boolean"
  ) {
    return document.createTextNode(String(content));
  } else {
    const container = document.createDocumentFragment();
    for (const child of content) {
      container.appendChild(renderChild(child));
    }
    return container;
  }
};

const componentsByName = new Map();
const namesByComponent = new Map();
export const createElement = <Props extends {} = {}>(
  type: string | FC<Props>,
  props: Props,
  ...children: Array<Renderable>
): HTMLElement => {
  const nativeProps = { ...(props || {}) };
  const style = nativeProps.style;
  delete nativeProps.style;

  let el: HTMLElement;
  if (typeof type === "string") {
    el = document.createElement(type);
    Object.assign(el, nativeProps);
    el.appendChild(renderChild(children));
  } else {
    let name = namesByComponent.get(type);
    if (!name) {
      const ownName = type.name || "_";
      let count = 1;
      name = ownName;

      while (componentsByName.has(name)) {
        count += 1;
        name = ownName + count;
      }

      if (type.name && count > 1) {
        console.warn(
          `Multiple (${count}) components named ${type.name} in use.`,
        );
      }

      componentsByName.set(name, type);
      namesByComponent.set(type, name);

      if (type.style) {
        const el = document.querySelector("style")!;
        const sheet: CSSStyleSheet = el.sheet as any;
        const i = sheet.insertRule(
          `body .${name}.Component {}`,
          sheet.rules.length,
        );
        Object.assign(sheet.rules[i].style, type.style);
        // HACK: these changes will work but will be confusingly invisible in the DOM so we write them back:
        el.textContent = Array.from(sheet.rules)
          .map(r => r.cssText)
          .join("\n");
      }
    }
    el = type(Object.assign(nativeProps, { children }));
    if (type.name) {
      el.classList.add(name, "Component");
    }
  }

  if (style) {
    Object.assign(el.style, style);
  }

  return el;
};

export const fragment = (
  ...fragments: Array<DocumentFragment | Element>
): DocumentFragment | Element => {
  if (fragments.length === 1) {
    return fragments[0];
  }
  const result = document.createDocumentFragment();
  for (const fragment of fragments) {
    result.appendChild(fragment);
  }
  return result;
};

Object.assign(window, {
  JSX: {
    createElement,
  },
});
