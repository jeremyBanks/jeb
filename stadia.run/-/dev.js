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

  root.querySelector(".loaded-games-status").textContent = 0;
  root.querySelector(".loaded-skus-status").textContent = 0;

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

  root.querySelector(".do-load-from-dev").disabled = !(await canFetchDevApi);
  root.querySelector(".do-save-to-dev").disabled = !(await canFetchDevApi);
  root.querySelector(
    ".do-load-from-store",
  ).disabled = !(await canFetchStadiaStore);
};

export const initialized = Promise.resolve()
  .then(() => {
    console.group("🔧 initializing dev tools");
    return init();
  })
  .finally(() => {
    console.groupEnd();
  });
