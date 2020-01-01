import React from "react";
import ReactDOM from "react-dom";

import { App } from "./core/main";

const main = async () => {
  const app = new App();
  const root = document.querySelector("main");
  const main = app.main();

  let done = false;
  while (!done) {
    ReactDOM.render(<React.StrictMode>{app.render()}</React.StrictMode>, root);
    done = await Promise.race([
      main.then(() => true),
      new Promise<false>((resolve) =>
        requestAnimationFrame(() => resolve(false))
      )
    ]);
  }
};

main();
