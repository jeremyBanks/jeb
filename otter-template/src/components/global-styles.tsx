import * as emotionReact from "@emotion/core";
import React from "react";

/**
 * Global style definitions that are applied throughout the entire document.
 */
export let GlobalStyles = () => (
  <emotionReact.Global
    styles={emotionReact.css({
      "input, textarea, button, select": {
        fontSize: "inherit"
      },

      "code, pre, kbd": {
        fontSize: "inherit",
        whiteSpace: "pre-wrap"
      },

      html: {
        boxSizing: "border-box"
      },

      body: {
        padding: "0",
        margin: "0",
        fontSize: "16px"
      },

      "*": {
        boxSizing: "inherit",
        fontFamily: "inherit",
        transitionDuration: "inherit"
      },

      a: {
        color: "blue",

        ":hover": {
          color: "red"
        }
      },

      "body > section": {
        display: "contents"
      },

      "body > section > *": {
        transitionDuration: "0.5s"
      },

      img: {
        verticalAlign: "middle"
      }
    })}
  />
);
