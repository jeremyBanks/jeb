import { css } from "emotion";
import React from "react";

import { SplashyGreeting } from "../components/splashy-greeting";
import { sleep } from "../utilities/async";
import { input, inputs } from "../utilities/input";
import { clamp, roundDown } from "../utilities/math";
import { print } from "../utilities/print";

export class App {
  /**
   * The top-level entry point running our application. You can use `print` to
   * output any type of content at the bottom of the page.
   */
  async main() {
    // Set the page's title and icon.
    const title = document.head.querySelector("title");
    const icon: HTMLLinkElement = document.head.querySelector("link[rel=icon]");
    title.textContent = "A Pup's Project";
    icon.href = "/icon.png";

    print("What is your name?");
    const name = await input("John Smith");

    print("Great! Just a moment...");

    await sleep(2.0);

    print(<SplashyGreeting name={name} />);

    do {
      print("Please accept our terms of service.");
    } while (
      "accept" !==
      (await input(
        <>
          <button autoFocus>accept</button>
          <button>reject</button>
        </>
      ))
    );

    await sleep(0.5);

    print(
      await inputs(
        <>
          <input
            name="something else"
            placeholder="hmm"
            required
            minLength={1}
          />
          <button autoFocus>A</button>
          <button>B</button>
          <button>C</button>
        </>
      )
    );

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
            height: 100
          })}
        >
          It has been <code>{seconds.toFixed(0).padStart(3, "0")}</code> seconds
          and this is displayed at a font size of <code>{fontSize}</code>{" "}
          pixels.
        </div>
      </>
    );
  }
}
