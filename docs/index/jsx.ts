export type Renderable = JSX.Renderable;

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

export const createElement = (
  type: string | ((props: Record<string, Renderable>) => HTMLElement),
  props: Record<string, any>,
  ...children: Array<Renderable>
): HTMLElement => {
  if (typeof type === "string") {
    const el = document.createElement(type);
    Object.assign(el, props);
    el.appendChild(renderChild(children));
    return el;
  } else {
    return type(Object.assign(props, { children }));
  }
};

Object.assign(window, {
  JSX: {
    createElement,
  },
});
