import { css, keyframes } from "emotion";
import React from "react";

export let SplashyGreeting = ({ name }: { name: string }) => {
  return (
    <div
      className={css({
        transition: "none",
        animationName: keyframes({
          "0%": {
            background: "lime"
          },
          "10%": {
            background: "magenta"
          },
          "50%": {
            background: "magenta"
          },
          "60%": {
            background: "lime"
          },
          "100%": {
            background: "lime"
          }
        }),
        animationDuration: "6s",
        animationIterationCount: "4",
        background: "lime",
        color: "black",
        fontWeight: 900,
        fontSize: "32px",
        padding: "32px"
      })}
    >
      <p>Welcome, {name}!</p>
    </div>
  );
};
