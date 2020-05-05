export type Renderable =
  | Element
  | DocumentFragment
  | Node
  | string
  | number
  | boolean
  | null
  | undefined
  | Iterable<Renderable>;

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

export const render = (
  type: string | ((props: Record<string, Renderable>) => Element),
  props: Record<string, any>,
  ...children: Array<Renderable>
): Element => {
  if (typeof type === "string") {
    const el = document.createElement(type);
    Object.assign(el, props);
    el.appendChild(renderChild(children));
    return el;
  } else {
    return type(Object.assign(props, { children }));
  }
};
