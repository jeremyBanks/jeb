#!/usr/bin/env -S deno run --allow-all
import SQL, { Database } from "https://deno.land/x/lite@0.0.9/sql.ts";

// https://dev.to/banks/chrome-cookies-deno-windows-wsl-wasm-aes-gcm-256-1fnb?preview=561cb24eeef17a834abcf3a6c227bbd1185380400b6704ab641113be6652abc3705d3fdad42ddf5ef94eb23455ee337978cc7d3a6ad77cabe5e107d6

const userData = "/mnt/c/Users/_/AppData/Local/Google/Chrome/User Data";

const localState = JSON.parse(
  await Deno.readTextFile(`${userData}/Local State`),
);

const profileInfoCache: Record<string, Record<string, any>> =
  localState["profile"]["info_cache"];

const encryptedChromeKey: string = localState["os_crypt"]["encrypted_key"];

for (const profileName of Object.keys(profileInfoCache)) {
  const cookiePath = `${userData}/${profileName}/Cookies`;
  const db = new Database(cookiePath);
  const rows = await db.query(SQL`SELECT host_key, name, encrypted_value from cookies`);


}

import init, {
  aes_gcm_256_decrypt_and_verify_as_utf8,
} from "./aes-gcm-256-wasm/pkg/aes_gcm_256_wasm.js";

await init();
