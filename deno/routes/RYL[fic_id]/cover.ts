export const config = {
  routeOverride: "/:fic_id(RYL[0-9A-Z]{7})(/cover)?.png",
};

import { Handlers } from "$fresh/server.ts";

const png = await Deno.readFile("static/cover.png");

export const handler: Handlers = {
  GET(_request, context) {
    const fic_id = context.params.fic_id;

    const body: Uint8Array = structuredClone(png);

    const crc = crc32(new TextEncoder().encode(String(fic_id)));

    const view = new DataView(body.buffer);

    // https://www.w3.org/TR/2003/REC-PNG-20031110/#5Chunk-layout
    // https://www.nayuki.io/page/png-file-chunk-inspector

    // https://www.w3.org/TR/2003/REC-PNG-20031110/#11PLTE
    const PLTE = body.subarray(0x0043, 0x0058);
    const PLTE_length = view.getUint32(PLTE.byteOffset);
    if (6 !== PLTE_length) throw new Error("internal consistency error");
    const PLTE_type = new TextDecoder().decode(PLTE.subarray(0x04, 0x08));
    if ("PLTE" !== PLTE_type) throw new Error("internal consistency error");
    const PLTE_body = body.subarray(
      PLTE.byteOffset + 0x04,
      PLTE.byteOffset + 0x08 + PLTE_length,
    );
    const PLTE_data = body.subarray(
      PLTE.byteOffset + 0x08,
      PLTE.byteOffset + 0x08 + PLTE_length,
    );
    const PLTE_crc = view.getUint32(PLTE.byteOffset + 0x08 + PLTE_length);
    if (crc32(PLTE_body) !== PLTE_crc) {
      throw new Error("internal consistency error");
    }

    const background = new PaletteColor(
      PLTE_data.subarray(0x00, 0x03),
    );
    const foreground = new PaletteColor(
      PLTE_data.subarray(0x03, 0x06),
    );

    const bytes = new Uint8Array(4);
    new DataView(bytes.buffer).setUint32(0, crc);

    if (bytes[0] & 0b101) {
      foreground.r ^= bytes[0] & 0b1111_1111;
      foreground.g ^= bytes[1] & 0b1111_1111;
      foreground.b ^= bytes[2] & 0b1111_1111;
    }

    if (~bytes[0] & 0b001) {
      background.r ^= bytes[0] & 0b1111_1111;
      background.g ^= bytes[1] & 0b1111_1111;
      background.b ^= bytes[2] & 0b1111_1111;
    }

    view.setUint32(PLTE.byteOffset + 0x08 + PLTE_length, crc32(PLTE_body));
    return Promise.resolve(
      new Response(body, {
        status: 200,
        headers: {
          "Content-Type": "image/png",
          "Cache-Control":
            "max-age=86400, stale-while-revalidate=31536000, stale-if-error=31536000",
        },
      }),
    );
  },
};

type Color = { r?: number; g?: number; b?: number };

class PaletteColor implements Color {
  constructor(
    protected rgbView: Uint8Array,
  ) {}
  get r() {
    return this.rgbView[0];
  }
  set r(r) {
    this.rgbView[0] = r;
  }
  get g() {
    return this.rgbView[1];
  }
  set g(g) {
    this.rgbView[1] = g;
  }
  get b() {
    return this.rgbView[2];
  }
  set b(b) {
    this.rgbView[2] = b;
  }
  toString() {
    return `0x${
      [this.r, this.g, this.b].map((x) => x.toString(16).padStart(2, "0")).join(
        "_",
      ).toUpperCase()
    }`;
  }
}

const u32 = (n: number): number => n >>> 0;
const crcPL = u32(0xEDB88320);
const crcIV = u32(0xFFFFFFFF);
const crcLK: Array<number> = (() => {
  const crcTable = new Array(256);
  for (let i = 0; i < 256; i++) {
    let c = i;
    for (let k = 0; k < 8; k++) {
      if (c & 1) {
        c = u32(crcPL ^ (c >>> 1));
      } else {
        c = c >>> 1;
      }
    }
    crcTable[i] = c;
  }
  return crcTable;
})();
const crc32 = (bytes: Uint8Array): number => {
  let crc = crcIV;
  for (const byte of bytes) {
    crc = u32((crc >>> 8) ^ crcLK[u32(crc ^ byte) & 0xFF]);
  }
  return u32(crc ^ crcIV);
};
