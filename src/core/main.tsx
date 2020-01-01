import { css } from "emotion";
import React from "react";

import { GlobalStyles } from "../components/global-styles";
import { SplashyGreeting } from "../components/splashy-greeting";
import { sleep } from "../utilities/async";
import { formInput, input, inputs } from "../utilities/input";
import { print } from "../utilities/print";
import { randomChoice } from "../utilities/random";

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
      <section
        className={css({
          border: "3px dashed navy",
          padding: "16px",
          maxWidth: "720px",
          lineHeight: "1.75",

          h1: {
            fontSize: "32px",
            margin: 0
          },

          ol: {
            margin: 0,
            paddingLeft: 0,

            li: {
              margin: "8px 32px"
            }
          },

          "code, pre": {
            background: "rgba(0, 0, 0, 0.03)",
            border: "1px solid rgba(0, 0, 0, 0.03)",
            borderRadius: "4px",
            padding: "0 4px"
          },

          code: {
            userSelect: "all",
            cursor: "pointer"
          }
        })}
      >
        <h1>Getting Started</h1>
        <ol>
          <li>
            <a href="https://github.com/otter-academy/template/generate">
              Create a repository from this template on GitHub
            </a>
            , clone it to your computer, and <code>cd</code> into it in a Linux
            shell.
          </li>
          <li>
            Run <code>source ./yarn install</code> to install everything
            required for this project, including{" "}
            <a href="https://github.com/nvm-sh/nvm">Node Version Manager</a> and
            all dependencies.
          </li>
          <li>
            Open this project folder in{" "}
            <a href="https://code.visualstudio.com/">Visual Studio Code</a>.
            Install the recommended extensions if prompted.
          </li>
          <li>
            Run <code>./yarn start</code> in your shell to launch the local
            development server.
          </li>
          <li>
            You will probably want to do most of your work inside the{" "}
            <code>core/</code>, <code>components/</code>, and{" "}
            <code>utilities/</code> subdirectories of <code>src/</code>. A good
            starting point would be to delete the introductory text you're
            reading right now from <code>src/core/main.tsx</code>.
          </li>
          <li>
            You can run <code>./yarn pretty</code> to auto-format the code.
          </li>
          <li>
            Run <code>./yarn deploy</code> to install, build, and publish to the
            web using GitHub Pages.
          </li>
        </ol>
        <p>Some example code continues below.</p>
      </section>
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

    const cards = (await import("../data/magic/arena.json")).default;
    print(`Loaded ${cards.length} cards. Here's one!`);

    const card = randomChoice(cards);
    print(card);
    print(
      <img
        src={card.image_uri}
        alt={card.name}
        className={css({
          width: "128px"
        })}
      />
    );

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

    await this.doMagicCardThing();

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
                transition: "none",
                height: 32,
                width: 32,
                objectFit: "contain",
                // This is silly hack to replace images in CSS.
                // see https://stackoverflow.com/a/37124764
                backgroundSize: "contain",
                backgroundImage: "url(/icon-shiny.png)",
                ":hover": {
                  objectPosition: "-65536px 65536px"
                }
              })}
            />
          </a>
          It has been <code>{seconds.toFixed(0).padStart(3, "0")}</code>{" "}
          seconds.
        </main>

        <GlobalStyles />
      </>
    );
  }

  async doMagicCardThing() {

    print(
      <p
        className={css({
          "code, pre": {
            background: "rgba(0, 0, 0, 0.03)",
            border: "1px solid rgba(0, 0, 0, 0.03)",
            borderRadius: "4px",
            padding: "0 4px"
          },

          code: {
            userSelect: "all",
            cursor: "pointer"
          }
        })}
      >
        Give us some files in{" "}
        <code>
          C:\Program Files (x86)\Wizards of the Coast\MTGA\MTGA_Data\Logs\Logs
        </code>
      </p>
    );

    const files = (
      await formInput(
        <>
          <input
            name="value"
            type="file"
            {...{ multiple: true, accept: ".log" }}
            className={css({
              "&:enabled": {
                cursor: "pointer"
              }
            })}
            onChange={(event) => {
              const el = Object.assign(document.createElement("button"), {
                hidden: true
              });
              event.target.form.appendChild(el);
              el.click();
              event.target.form.removeChild(el);
            }}
          />
        </>
      )
    ).getAll("value") as Array<File>;

    const contents = (
      await Promise.all(
        files.map(async (file) => ({
          name: file.name,
          lastModified: file.lastModified,
          text: new TextDecoder()
            .decode(
              await (file as {
                arrayBuffer(): Promise<ArrayBuffer>;
              }).arrayBuffer()
            )
        }))
      )
    ).sort((a, b) => a.lastModified - b.lastModified);

    for (const content of contents) {
      print("---");
      print(content.lastModified, content.name);
      const lines = content.text.split(/\n/g);
      for (const line of lines.slice(-32)) {
        print(line)
      }
    }
  }
}
