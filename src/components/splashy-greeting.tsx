import { css, keyframes } from "emotion";
import React from "react";

export const SplashyGreeting: React.FC<{ name: string }> = ({ name }) => {
  return (
    <div
      className={css({
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
        animationIterationCount: "infinite",
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
