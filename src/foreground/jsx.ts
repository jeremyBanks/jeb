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

const componentsByName = new Map();
const namesByComponent = new Map();
export const createElement = (
  type: string | ((props: Record<string, Renderable>) => HTMLElement),
  props: Record<string, any>,
  ...children: Array<Renderable>
): HTMLElement => {
  props = { ...(props || {}) };
  const style = props.style;
  delete props.style;

  let el: HTMLElement;
  if (typeof type === "string") {
    el = document.createElement(type);
    Object.assign(el, props);
    el.appendChild(renderChild(children));
  } else {
    let name = namesByComponent.get(type);
    if (!name) {
      const ownName = type.name || "_component";
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
    }
    el = type(Object.assign(props, { children }));
    if (type.name) {
      el.classList.add(name);
    }
  }

  if (style) {
    Object.assign(el.style, style);
  }

  return el;
};

Object.assign(window, {
  JSX: {
    createElement,
  },
});
