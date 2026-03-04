import { decode as z855Decode, encode as z855Encode } from "../z855-readable.ts";

// Read hex-encoded test data from stdin
const input = await Deno.readTextFile("/dev/stdin");
const lines = input.trim().split("\n");

for (const line of lines) {
    const [cmd, ...rest] = line.split(" ");
    if (cmd === "DECODE") {
        // Decode the Z855 string
        const encoded = rest.join(" ");
        try {
            const decoded = z855Decode(encoded);
            console.log(`OK ${decoded.length} ${Array.from(decoded).map(b => b.toString(16).padStart(2, '0')).join('')}`);
        } catch (e) {
            console.log(`ERR ${e.message}`);
        }
    } else if (cmd === "ENCODE") {
        // Encode hex bytes
        const hex = rest.join("");
        const bytes = new Uint8Array(hex.match(/.{2}/g)!.map(h => parseInt(h, 16)));
        const encoded = z855Encode(bytes);
        console.log(`OK ${encoded}`);
    }
}
