import * as emotionReact from "@emotion/core";
import { css } from "emotion";
import React from "react";

import { SplashyGreeting } from "../components/splashy-greeting";
import { sleep } from "../utilities/async";
import { input, inputs } from "../utilities/input";
import { print } from "../utilities/print";

export class App {
  /**
   * The top-level entry point running our application. You can use `print` to
   * output any type of content at the bottom of the page.
   */
  async main() {
    const icon = document.getElementById("icon") as HTMLLinkElement;
    icon.href = "/icon.png";

    document.title = "Your Project";

    print(
      <p>
        You can{" "}
        <a href="https://github.com/otter-academy/template/generate">
          create a repository from this template on GitHub
        </a>
        , edit it to your heart's content, then <code>./yarn deploy</code> it to
        the web.
      </p>
    );

    print("What is your name?");
    const name = await input("John Smith");

    print("Great! Just a moment...");

    await sleep(2.0);

    print(<SplashyGreeting name={name} />);

    for (let i = 0; i < 20; i++) {
      await sleep(0.1);
      print(i);
    }

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
            name="thinking"
            placeholder="what are you thinking?"
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
    let textShadow = "none";
    if (seconds > 2 && seconds < 8) {
      textShadow = "4px 6px 4px rgb(200, 200, 200)";
    }

    return (
      <>
        <main
          className={css({
            position: "sticky",
            top: "0px",
            padding: "16px",
            fontWeight: "bold",
            color: "rgb(0, 25, 100)",
            borderBottom: "3px solid black",
            boxShadow: "0 1px 3px rgba(0, 0, 0, 0.5)",
            background: "#FFFFF8",
            zIndex: 100,
            textShadow
          })}
        >
          <a href="/">
            <img
              alt=""
              src="/icon.png"
              className={css({
                height: 32,
                width: 32
              })}
            />
          </a>
          It has been <code>{seconds.toFixed(0).padStart(3, "0")}</code>{" "}
          seconds.
        </main>

        <emotionReact.Global
          styles={emotionReact.css({
            html: {
              boxSizing: "border-box"
            },

            a: {
              color: "blue",

              ":hover": {
                color: "red"
              }
            },

            "*": {
              boxSizing: "inherit",
              fontFamily: "inherit",
              transitionDuration: "inherit"
            },

            body: {
              padding: "0",
              margin: "0",
              fontSize: "16px"
            },

            "body > section": {
              display: "contents"
            },

            "body > section > *": {
              transitionDuration: "0.5s"
            },

            img: {
              verticalAlign: "middle"
            },

            "code, pre, kbd": {
              whiteSpace: "pre-wrap"
            }
          })}
        />
      </>
    );
  }
}
