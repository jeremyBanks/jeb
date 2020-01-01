/**
 * Returns a promise that is resolved after waiting for a number of seconds.
 */
export let sleep = (seconds: number): Promise<void> =>
  new Promise((resolve) => setTimeout(resolve, seconds * 1000));

/**
 * A promise that will never settle/resolve/reject; it will be pending forever.
 */
export let NEVER: Promise<never> = new Promise((_resolve, _reject) => {});
