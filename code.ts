type bool = boolean;
type f64 = number;
type u64 = bigint;
const u64  = (i: bigint | string): u64 => {
    const u = BigInt(i);
    if (u < 0n || u > 0xFFFFFFFFFFFFFFFF) {
        throw new TypeError(`u64 out of bounds: ${u}`);
    }
    return u;
}
u64.toBase64 = (i: bigint): string => "0";
u64.tryFromBase64 = (s: string): u64 | undefined => {
    if (s.length !== 11) {
        return undefined;
    }
};
u64.tryFromBase16 = (s: string): u64 | undefined => 0n;
u64.tryFromBase10 = (s: string): u64 | undefined => {
    const u = BigInt(i);
    if (u < 0n || u > 0xFFFFFFFFFFFFFFFF) {
        throw new TypeError(`u64 out of bounds: ${u}`);
    }
    return u;
};
u64.tryFrom = (s: string): u64 | undefined =>
    u64.tryFromBase64(s) ??
    u64.tryFromBase16(s) ??
    u64.tryFromBase10(s);


// ensure URLs are always their canonical representation
// and that that is simple ascii

// youtube uses: -_
// javascript uses: /+
// does removing the padding allow some kind of collission?

class Spidered {
    // the timestamp at which we first saw a reference to this record
    firstRef: f64 | undefined = undefined
    // the timestamp at which we first saw this record directly
    firstLoaded: f64 | undefined = undefined
    // the timestamp at which we first saw that this record was removed
    firstGone: f64 | undefined = undefined
    // the timestamp at which we most recently saw a reference to this record
    lastRef: f64 | undefined = undefined
    // the timestamp at which we we most recently saw this record directly
    lastLoaded: f64 | undefined = undefined
    // the timestamp at which we most recently saw that this record was removed
    lastGone: f64 | undefined = undefined

    listed() {
        if (this.lastGone === undefined) {
            // it has never been delisted
            return true;
        } else if (
            this.lastLoaded && this.lastLoaded > this.lastGone ||
            this.lastRef && this.lastRef > this.lastGone
        ) {
            // it was delisted but we've seen it restored
            return true;
        } else {
            // it is delisted
            return false;
        }
    }

    markRef() {
        let now = Date.now();
        if (this.firstRef === undefined || this.firstRef > now) {
            this.firstRef = now;
        }
        if (this.lastRef === undefined || this.lastRef < now) {
            this.lastRef = now;
        }
    }

    markGone() {
        let now = Date.now();
        if (this.firstGone === undefined || this.firstGone > now) {
            this.firstGone = now;
        }
        if (this.lastGone === undefined || this.lastGone < now) {
            this.lastGone = now;
        }
    }

    load(source: Readonly<Partial<this>>) {
        let now = Date.now();
        if (this.firstLoaded === undefined || this.firstLoaded > now) {
            this.firstLoaded = now;
        }
        if (this.lastLoaded === undefined || this.lastLoaded < now) {
            this.lastLoaded = now;
        }
        for (let key of Object.getOwnPropertyNames(source) as (keyof this)[]) {
            let value = source[key];
            if (value !== undefined) {
                this[key] = value as any;
            }
        }
    }
}

class Player extends Spidered {

}

class Game extends Spidered {

}
