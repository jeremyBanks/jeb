import { css, keyframes } from "emotion";
import React, { ReactNode } from "react";

/**
 * Renders a value that's been printed, with an animation to draw attention to the change.
 */
export const Printed: React.FC<{ values: Array<ReactNode | any> }> = ({
  values
}) => (
  <p className={printed}>
    {values.map((value, i) => {
      if (
        React.isValidElement(value) ||
        typeof value == "string" ||
        typeof value == "number"
      ) {
        return <span key={i}>value</span>;
      } else {
        try {
          return <code key={i}>{JSON.stringify(value, null, 2)}</code>;
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
  </p>
);

const printed = css({
  textShadow: "0 0 0 rgba(0, 0, 0, 0)",
  color: "rgba(0, 0, 0, 1.0)",
  animationName: `${keyframes({
    "0%": {
      textShadow: "0 0 16px rgba(0, 0, 0, 0)",
      color: "rgba(0, 0, 0, 0)",
      opacity: 0.0
    },
    "5%": {
      textShadow: "0 0 8px black",
      color: "rgba(0, 0, 0, 0)",
      opacity: 1.0
    },
    "25%": {
      textShadow: "0 0 4px black",
      color: "rgba(0, 0, 0, 0.5)",
      opacity: 1.0
    },
    "100%": {
      textShadow: "0 0 32px rgba(0, 0, 0, 0)",
      color: "rgba(0, 0, 0, 1.0)",
      opacity: 1.0
    }
  })}`,
  animationDuration: "0.5s",
  animationTimingFunction: "linear",
  animationIterationCount: 1,
  animationFillMode: "forwards"
});
