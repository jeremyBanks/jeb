import * as log from "https://deno.land/std@0.78.0/log/mod.ts";

import { discoverProfiles } from "./chrome/mod.ts";

await log.setup({
  handlers: {
    console: new log.handlers.ConsoleHandler("DEBUG"),
  },
  loggers: {
    default: {
      level: "DEBUG",
      handlers: ["console"],
    },
  },
});

const profiles = await discoverProfiles();

log.debug(`Discovered ${profiles.length} Chrome profiles.`);

for (const profile of profiles) {
  log.info(`${profile}`);

  try {
    console.log((await profile.cookies())[0]);
  } catch (error) {
    console.error(error);
    throw error;
  }
}
