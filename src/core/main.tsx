import { css } from "emotion";
import React from "react";

import { SplashyGreeting } from "../components/splashy-greeting";
import { sleep } from "../utilities/async";
import { clamp, roundDown } from "../utilities/math";
import { print } from "../utilities/print";

export class App {
  /**
   * The top-level entry point running our application. You can use `print` to
   * output any type of content at the bottom of the page.
   */
  async main() {
    const title = document.head.querySelector("title");
    title.textContent = "A Pup's Project";

    const icon: HTMLLinkElement = document.head.querySelector("link[rel=icon]");
    icon.href = "/icon.png";

    print("hello world");
    print(
      <p>
        <b>hello</b>{" "}
        <i
          className={css({
            color: "green"
          })}
        >
          world
        </i>
        !
      </p>
    );

    let x = 0;
    while (true) {
      print("What do I know?", { x });
      await sleep(2);
      x += 1;

      if (!"x".startsWith("x")) {
        break;
      }
    }

    // keep running forever
    return new Promise(() => {});
  }

  /**
   * Generates the interface of our application that we want to display every
   * frame for as long as main() is running. This is displayed at the top of
   * the page, above any `print()` output.
   *
   * Please note that normally, React components only update if their properties
   * or state are changed in a way that React is directly aware of, but we are
   * manually forcing a re-render every frame here to allow us to ignore those
   * elements of React initially.
   */
  render(): React.ReactNode {
    const seconds = performance.now() / 1000;
    const fontSize = clamp(20, roundDown(16 + seconds), 30);
    let textShadow = "none";
    if (seconds > 2 && seconds < 8) {
      textShadow = "4px 6px 4px rgb(200, 200, 200)";
    }

    return (
      <>
        <div
          className={css({
            fontWeight: "bold",
            fontSize,
            color: "rgb(0, 25, 100)",
            textShadow,
            height: 300
          })}
        >
          It has been <code>{seconds.toFixed(0).padStart(3, "0")}</code> seconds
          and this is displayed at a font size of <code>{fontSize}</code>{" "}
          pixels.
        </div>

        <SplashyGreeting name="Padawan Learner" />
      </>
    );
  }
}
