/**
 * @returns {Promise<unknown>}
 */
export const sleep = (seconds = 0) =>
  new Promise(resolve => setTimeout(resolve, seconds * 1000));

/**
 * @param {number} seconds
 * @param {Promise<T>} promise
 * @returns {Promise<T>}
 * @template T
 */
export const withTimeout = (seconds, promise) => {
  return Promise.race([
    new Promise((_, reject) => {
      sleep(seconds).then(reject);
    }),
    promise,
  ]);
};
