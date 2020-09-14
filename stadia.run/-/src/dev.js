// @ts-nocheck WIP

import { spiderThread } from "./spider.js";
import {
  canFetchDevApi,
  canFetchStadiaHost,
  canFetchStadiaStore,
} from "./net.js";

const init = async () => {
  const root = document.getElementById("dev-tools");

  root.classList.remove("unloaded");
  document.querySelector("footer").classList.add("activated");

  canFetchStadiaStore.then(spiderThread);

  root.querySelector(".dev-server-status").textContent = (await canFetchDevApi)
    ? "✅ available"
    : "❌ unavailable";

  root.querySelector(
    ".stadia-proxy-status",
  ).textContent = (await canFetchStadiaStore)
    ? "✅ authenticated"
    : (await canFetchStadiaHost)
    ? "⚠️ unauthenticated"
    : "❌ unavailable";
};

export const initialized = Promise.resolve()
  .then(() => {
    console.group("🔧 initializing dev tools");
    return init();
  })
  .finally(() => {
    console.groupEnd();
  });
