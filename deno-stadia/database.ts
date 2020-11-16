#!/usr/bin/env -S deno run --allow-env --allow-net=127.0.0.1:57414,stadia.google.com --allow-read=/ --allow-write=/ --allow-run
const net = "127.0.0.1:57414";
const data = "./database.sqlite";

import SQL from "https://deno.land/x/lite@0.0.9/sql.ts";
import * as log from "https://deno.land/std@0.75.0/log/mod.ts";
import { serve } from "https://deno.land/std@0.77.0/http/server.ts";
import { assert } from "https://deno.land/std@0.75.0/testing/asserts.ts";

import init, {
  aes_gcm_256_decrypt_and_verify_as_utf8,
} from "./aes_gcm_256_decrypt_and_verify_as_utf8/mod.ts";

await init();

// https://stackoverflow.com/a/60423699

const chromeState = await Deno.readTextFile(
  "/mnt/c/Users/_/AppData/Local/Google/Chrome/User Data/Local State",
).then(JSON.parse);

const chromeProfiles = Object.entries(chromeState.profile.info_cache).filter((
  x: any,
) => x[1] && x[1].gaia_id)
  .sort((a: any, b: any) => b[1].active_time - a[1].active_time).map((
    [key, x]: any,
  ) => ({
    key,
    name: [...new Set([x.gaia_name, x.name])].join(" "),
    email: x.user_name || null,
    google_id: x.gaia_id as string,
    last_accessed_stadia: 0n,
  }));

console.log(chromeState.os_crypt.encrypted_key);

// AES-256-GCM key used for local data that is encrypted at rest.
const encryptedChromeKey = chromeState.os_crypt.encrypted_key as string;

const chromeKey = new Uint8Array(
  [...atob(String.fromCharCode(
    ...(await Deno.run({
      cmd: [
        "./dpapibridge.exe",
        "--decrypt",
        "--base64",
        "--input",
        btoa(atob(encryptedChromeKey).slice(5)),
      ],
      stdout: "piped",
    }).output()),
  ))].map((c) => c.codePointAt(0) ?? 0),
); //.slice(4, 4 + 32);

console.log(chromeKey);
log.info(
  `Chrome local decryption key: ${chromeKey}`,
);

const sessionCookieNames = ["HSID", "SSID", "SID"];

for (const profile of chromeProfiles) {
  const db = SQL(
    `/mnt/c/Users/_/AppData/Local/Google/Chrome/User Data/${profile.key}/Cookies`,
  );

  profile.last_accessed_stadia = (await db(
    SQL
      `SELECT last_access_utc from cookies where host_key = '.stadia.google.com' order by last_access_utc desc limit 0, 1`,
  ))[0]?.last_access_utc as bigint ?? 0n;
}

chromeProfiles.sort(
  (a: any, b: any) => Number(b.last_accessed_stadia - a.last_accessed_stadia),
);

const credentials: Record<string, any> = {};

for (const profile of chromeProfiles) {
  const db = SQL(
    `/mnt/c/Users/_/AppData/Local/Google/Chrome/User Data/${profile.key}/Cookies`,
  );

  if (!profile.last_accessed_stadia) {
    continue;
  }

  log.info(
    `${profile.name} <${profile.email}> last accessed stadia ${
      new Date(
        Number(profile.last_accessed_stadia.toString()) / 1000 -
          11644473600000,
      )
        .toISOString()
    }`,
  );
  const cookieCreds =
    Object.fromEntries(
      (await db(
        SQL
          `SELECT name, encrypted_value FROM cookies where host_key like '%.google.com' order by name asc`,
      )).filter((r) => sessionCookieNames.includes(r.name as string)).map(
        (
          x,
        ) => {
          const data = x.encrypted_value as unknown as Uint8Array;
          const nonce = data.slice(3, 15);
          const ciphertext = data.slice(15);
          return [
            x.name,
            aes_gcm_256_decrypt_and_verify_as_utf8(
              chromeKey,
              nonce,
              ciphertext,
            ),
          ];
        },
      ),
    );

  if (!(cookieCreds["HSID"] && cookieCreds["SSID"] && cookieCreds["SID"])) {
    log.warning("...but they don't currently have credentials saved.")
  } else {
    log.info("...and they appear to have saved credentials.")
    credentials[profile.email] = cookieCreds;
  }
}

const googleSession = credentials["nobody@jeremy.ca"];

const response = await fetch(
  "https://stadia.google.com/profile",
  {
    "headers": {
      "user-agent":
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/86.0.4240.198 Safari/537.36",
      "cookie":
        `HSID=${googleSession.HSID}; SSID=${googleSession.SSID}; SID=${googleSession.SID}`,
    },
    "body": null,
    "method": "GET",
    "mode": "cors",
  },
);

const body = await response.text();

// XXX: see no eval
const preloadRequests = eval('(' + (body.match(/AF_dataServiceRequests =(.+); var AF_initDataChunkQueue =/s)?.[1] ?? 'null') + ')');
const preloadResponses = [...body.matchAll(/>AF_initDataCallback(\(\{.*?\}\))\;<\/script>/gs)].map((x: any) => {
  return eval(x[1]);
});

const preloads = [];
for (const response of preloadResponses) {
  const request = preloadRequests[response.key];
  preloads.push({
    id: request.id,
    args: request.request,
    isError: response.isError,
    data: response.data,
  })
}

console.log(preloads);
