import { css, keyframes } from "emotion";
import React, { ReactNode } from "react";

/**
 * Renders a value that's been printed, with an animation to draw attention to the change.
 */
export let Printed = ({
  values,
  inline
}: {
  values: Array<ReactNode | any>;
  inline?: boolean;
}) => (
  <div
    className={css(
      printed,
      inline
        ? {
            display: "inline",
            margin: 2
          }
        : {}
    )}
  >
    {values.map((value, i) => {
      if (
        React.isValidElement(value) ||
        typeof value == "string" ||
        typeof value == "number"
      ) {
        return <span key={i}>{value}</span>;
      } else {
        if (value instanceof FormData) {
          value = Object.fromEntries(
            [...value.keys()].map((key) => [key, value.getAll(key)])
          );
        }
        try {
          let json = JSON.stringify(value, null, 2);
          if (json && json.length > 512) {
            json = json.slice(0, 512) + "… (truncated)";
          }
          return <code key={i}>{json}</code>;
        } catch (error) {
          console.error(error);
          return (
            <pre
              key={i}
              className={css({
                color: "maroon",
                padding: "8px",
                border: "3px double red",
                background: "white"
              })}
            >
              {error.toString()}
              {"\n"}
              {error.stack}
            </pre>
          );
        }
      }
    })}
  </div>
);

let printed = css({
  transition: "none",
  margin: 16,
  textShadow: "0 0 0 rgba(0, 0, 0, 0)",
  color: "rgba(0, 0, 0, 1.0)",
  animationName: `${keyframes({
    "0%": {
      textShadow: "0 0 16px rgba(0, 0, 0, 0)",
      color: "rgba(0, 0, 0, 0)",
      opacity: 0.0
    },
    "40%": {
      textShadow: "0 0 4px black",
      color: "rgba(0, 0, 0, 0)",
      opacity: 1.0
    },
    "60%": {
      textShadow: "0 0 4px rgba(0, 0, 0, 0.25)",
      color: "rgba(0, 0, 0, 0.5)",
      opacity: 1.0
    },
    "100%": {
      textShadow: "0 0 32px rgba(0, 0, 0, 0)",
      color: "rgba(0, 0, 0, 1.0)",
      opacity: 1.0
    }
  })}`,
  animationDuration: "0.25s",
  animationTimingFunction: "ease-in-out",
  animationIterationCount: 1,
  animationFillMode: "forwards"
});
