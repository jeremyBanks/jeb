import "./foreground/autoreload.js";
import "./foreground/jsx.js";

import * as models from "./foreground/data/models.js";
import * as views from "./foreground/views.js";

import { fragment } from "./foreground/jsx.js";
import { localKey } from "./foreground/data/local-key.js";

export const main = async () => {
  document.head.appendChild(
    fragment(
      <title>stadia.observer</title>,
      <link rel="icon" href="/illufinch-violetsky-edited@2x.png" />,
    ),
  );

  // TODO: associate prototypes, these are plain Objects.
  const skus: Array<models.Sku> = Object.values(
    await (await fetch("/skus.json")).json(),
  );

  // re-sort in case we changed the key function
  skus.sort((a, b) => {
    let ap = localKey(a);
    let bp = localKey(b);
    if (ap < bp) {
      return -1;
    } else if (bp < ap) {
      return +1;
    } else {
      return 0;
    }
  });

  document.body.appendChild(<views.Home skus={skus} />);
};

main().catch(error => console.error("Unexpected error in main.", error));
