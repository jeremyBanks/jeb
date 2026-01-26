import { Options } from "$fresh/plugins/twind.ts";
import { Plugin } from "$fresh/server.ts";
import { asset } from "$fresh/runtime.ts";
import * as colors from "twind/colors";
import { apply } from "twind/css#css_directive";

export default {
  selfURL: import.meta.url,
  theme: {
    extend: {
      colors,
      fontFamily: {
        mono: ["'JetBrains Mono'", "ui-monospace", "monospace"],
        sans: ["'Atkinson Hyperlegible'", "ui-sans-serif", "sans-serif"],
        serif: ["Georgia", "ui-serif", "serif"],
      },
      spacing: {
        "96": "24rem",
        "128": "32rem",
        "192": "48rem",
      },
    },
  },
  preflight: (preflight) => ({
    ...preflight,
    "html, body":
      apply`flex justify-center items-center h-full w-full bg-white text-black font-sans`,
    code: apply`font-mono`,
    pre: apply`font-mono`,
    "@font-face": [
      {
        fontFamily: "Atkinson Hyperlegible",
        fontStyle: "normal",
        fontWeight: 400,
        fontDisplay: "fallback",
        src: `url(${asset("/fonts/sans400.woff2")}) format("woff2")`,
      },
      {
        fontFamily: "Atkinson Hyperlegible",
        fontStyle: "italic",
        fontWeight: 400,
        fontDisplay: "fallback",
        src: `url(${asset("/fonts/sans401.woff2")}) format("woff2")`,
      },
      {
        fontFamily: "Atkinson Hyperlegible",
        fontStyle: "normal",
        fontWeight: 700,
        fontDisplay: "fallback",
        src: `url(${asset("/fonts/sans700.woff2")}) format("woff2")`,
      },
      {
        fontFamily: "Atkinson Hyperlegible",
        fontStyle: "italic",
        fontWeight: 700,
        fontDisplay: "fallback",
        src: `url(${asset("/fonts/sans701.woff2")}) format("woff2")`,
      },
      {
        fontFamily: "JetBrains Mono",
        fontStyle: "normal",
        fontWeight: 400,
        fontDisplay: "fallback",
        src: `url(${asset("/fonts/mono400.woff2")}) format("woff2")`,
      },
      {
        fontFamily: "JetBrains Mono",
        fontStyle: "italic",
        fontWeight: 400,
        fontDisplay: "fallback",
        src: `url(${asset("/fonts/mono401.woff2")}) format("woff2")`,
      },
      {
        fontFamily: "JetBrains Mono",
        fontStyle: "normal",
        fontWeight: 700,
        fontDisplay: "fallback",
        src: `url(${asset("/fonts/mono700.woff2")}) format("woff2")`,
      },
      {
        fontFamily: "JetBrains Mono",
        fontStyle: "italic",
        fontWeight: 700,
        fontDisplay: "fallback",
        src: `url(${asset("/fonts/mono701.woff2")}) format("woff2")`,
      },
    ],
  }),
} as Options;
