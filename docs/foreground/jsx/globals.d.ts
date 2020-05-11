declare namespace JSX {
  const createElement: (
    type: string | ((props: Record<string, Renderable>) => HTMLElement),
    props: Record<string, any>,
    ...children: Array<Renderable>
  ) => HTMLElement;

  type Renderable =
    | HTMLElement
    | DocumentFragment
    | Node
    | string
    | number
    | boolean
    | null
    | undefined
    | Iterable<Renderable>;

  interface IntrinsicElements {
    a: _Element<HTMLAnchorElement>;
    section: _Element;
    main: _Element<HTMLElement>;
    div: _Element<HTMLDivElement>;
    span: _Element<HTMLSpanElement>;
    hr: _Element<HTMLHRElement>;
    p: _Element<HTMLParagraphElement>;
    button: _Element<HTMLButtonElement>;
    h1: _Element<HTMLHeadingElement>;
    h2: _Element<HTMLHeadingElement>;
    h3: _Element<HTMLHeadingElement>;
    h4: _Element<HTMLHeadingElement>;
    h5: _Element<HTMLHeadingElement>;
    h6: _Element<HTMLHeadingElement>;
    h7: _Element<HTMLHeadingElement>;
  }

  // Properties that our renderer handles in a way inconsistent with the underlying HTML DOM interface.
  type _OverrideProps = {
    children: Renderable;
    style: Partial<CSSStyleDeclaration>;
  };

  type _Element<T extends HTMLElement = HTMLElement> = Partial<
    Omit<T, keyof _OverrideProps> & _OverrideProps
  >;

  interface ElementChildrenAttribute {
    children: Renderable;
  }
}
