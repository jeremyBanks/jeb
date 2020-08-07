
class Spidered {
    // the timestamp at which we first saw a reference to this record
    firstRef: number | undefined = undefined
    // the timestamp at which we first saw this record directly
    firstLoaded: number | undefined = undefined
    // the timestamp at which we first saw that this record was removed
    firstGone: number | undefined = undefined
    // the timestamp at which we most recently saw a reference to this record
    lastRef: number | undefined = undefined
    // the timestamp at which we we most recently saw this record directly
    lastLoaded: number | undefined = undefined
    // the timestamp at which we most recently saw that this record was removed
    lastGone: number | undefined = undefined

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
        const now = Date.now();
        if (this.firstRef === undefined || this.firstRef > now) {
            this.firstRef = now;
        }
        if (this.lastRef < now) {
            this.lastRef = now;
        }
    }
    
    markGone() {
        const now = Date.now();
        if (this.firstGone === undefined || this.firstGone > now) {
            this.firstGone = now;
        }
        if (this.lastGone < now) {
            this.lastGone = now;
        }
    }

    load(source: Readonly<Partial<this>>) {
        const now = Date.now();
        if (this.firstLoaded === undefined || this.firstLoaded > now) {
            this.firstLoaded = now;
        }
        if (this.lastLoaded < now) {
            this.lastLoaded = now;
        }
        for (const key of Object.getOwnPropertyNames(source)) {
            const value = source[key];
            if (value !== undefined) {
                this[key] = value;
            }
        }
    }
}

class Player extends Spidered {

}

class Game extends Spidered {

}

