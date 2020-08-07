declare namespace JSX {
  type FC<Props extends {} = {}> = ((
    props: Props & { children?: Array<Renderable> },
  ) => HTMLElement) & {
    style?: Partial<CSSStyleDeclaration>;
  };

  const createElement: <Props extends {} = {}>(
    type: string | FC<Props>,
    props: Props,
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
    button: _Element<HTMLButtonElement>;
    code: _Element;
    div: _Element<HTMLDivElement>;
    h1: _Element<HTMLHeadingElement>;
    h2: _Element<HTMLHeadingElement>;
    h3: _Element<HTMLHeadingElement>;
    h4: _Element<HTMLHeadingElement>;
    h5: _Element<HTMLHeadingElement>;
    h6: _Element<HTMLHeadingElement>;
    h7: _Element<HTMLHeadingElement>;
    hr: _Element<HTMLHRElement>;
    link: _Element<HTMLLinkElement>;
    main: _Element<HTMLElement>;
    p: _Element<HTMLParagraphElement>;
    pre: _Element<HTMLPreElement>;
    section: _Element;
    span: _Element<HTMLSpanElement>;
    title: _Element<HTMLTitleElement>;
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
