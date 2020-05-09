declare namespace JSX {
  interface ElementChildrenAttribute {
    children: any;
  }
  interface IntrinsicElements {
    [elemName: string]: {
      [propName: string]: any;
      children: any;
    };
  }
}
