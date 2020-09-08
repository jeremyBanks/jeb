/** @returns {Promise<unknown>} */
export const sleep = (seconds = 0) =>
  new Promise(resolve => setTimeout(resolve, seconds * 1000));
