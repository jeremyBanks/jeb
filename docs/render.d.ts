export declare type Renderable =
  | Element
  | DocumentFragment
  | Node
  | string
  | number
  | boolean
  | null
  | undefined
  | Iterable<Renderable>;
export declare const render: (
  type: string | ((props: Record<string, Renderable>) => Element),
  props: Record<string, any>,
  ...children: Renderable[]
) => Element;
