import React, { ReactNode } from "react";
import ReactDOM from "react-dom";

import { Printed } from "../components/printed";

export let printInline = (...values: Array<ReactNode | unknown>) =>
  print(print.inline, ...values);

/**
 * Prints one or more values at the bottom of the document. Scrolling sticks to
 * he bottom of the document unless the user how scrolled up.
 *
 * The values may be of any type, and will be properly rendered if they are
 * DOM elements, React components, or if they implement the `[print.as]` method.
 * Otherwise they will be displayed as text or JSON.
 */
export let print = Object.assign(
  (...values: Array<ReactNode | unknown>): void => {
    let inline = false;
    if (values[0] === print.inline) {
      inline = true;
      values.shift();
    }
    console.debug(...values);

    let oldMax = document.documentElement.scrollHeight - window.innerHeight;
    let atBottom = document.documentElement.scrollTop >= oldMax - 4;

    let container = document.createElement("article");
    container.style.display = "contents";
    document.getElementById("prints").appendChild(container);

    ReactDOM.render(
      <React.StrictMode>
        <Printed
          inline={inline}
          values={values.flatMap((value) => {
            while (typeof value?.[print.as] === "function") {
              value = value[print.as]();
            }
            return value;
          })}
        />
      </React.StrictMode>,
      container
    );

    let newMax = document.documentElement.scrollHeight - window.innerHeight;

    if (atBottom) {
      document.documentElement.scrollTop = newMax;
    }
  },
  {
    /**
     * Symbolic method name used to override display of a printed value.
     *
     * This only applies to values passed-in to print directly, it won't affect any
     * internal rendering that takes place within the values.
     */
    as: Symbol("print.as"),

    inline: Symbol("print.inline")
  }
);
