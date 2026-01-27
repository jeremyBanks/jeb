import React from "react";
import ReactDOM from "react-dom";

import { App } from "./core/main";

let main = async () => {
  let app = new App();
  let root = document.getElementById("main");
  let main = app.main();

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

  console.debug("main() has exited.");
};

main();
