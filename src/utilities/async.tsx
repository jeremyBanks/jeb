/**
 * Returns a promise that is resolved after waiting for a number of seconds.
 */
export const sleep = (seconds: number): Promise<void> =>
  new Promise((resolve) => setTimeout(resolve, seconds * 1000));
