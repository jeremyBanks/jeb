import React, { ReactNode } from "react";
import ReactDOM from "react-dom";

import { Printed } from "../components/printed";

/**
 * Prints one or more values at the bottom of the document. Scrolling sticks to
 * he bottom of the document unless the user how scrolled up.
 *
 * The values may be of any type, and will be properly rendered if they are
 * DOM elements, React components, or if they implement the `[print.as]` method.
 * Otherwise they will be displayed as text or JSON.
 */
export const print = (...values: Array<ReactNode | unknown>): void => {
  console.log(...values);

  const oldMax =
    window.document.documentElement.scrollHeight - window.innerHeight;
  const atBottom = window.document.documentElement.scrollTop >= oldMax - 4;

  const container = document.createElement("div");

  ReactDOM.render(
    <React.StrictMode>
      <Printed
        values={values.flatMap((value, i) => {
          while (typeof value?.[print.as] === "function") {
            value = value[print.as]();
          }
          return value;
        })}
      />
    </React.StrictMode>,
    container
  );

  document.body.appendChild(container);

  const newMax =
    window.document.documentElement.scrollHeight - window.innerHeight;

  if (atBottom) {
    window.document.documentElement.scrollTop = newMax;
  }
};

/**
 * Symbolic method name used to override display of a printed value.
 *
 * This only applies to values passed-in to print directly, it won't affect any
 * internal rendering that takes place within the values.
 */
print.as = Symbol("print.as");
