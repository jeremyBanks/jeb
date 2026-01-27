/** @returns {Promise<unknown>} */
export const sleep = (seconds = 0) =>
  new Promise(resolve => setTimeout(resolve, seconds * 1000));

export const withTimeout = (seconds, promise) => {
  return Promise.race([
    new Promise((_, reject) => {
      sleep(seconds).then(reject);
    }),
    promise,
  ]);
};
