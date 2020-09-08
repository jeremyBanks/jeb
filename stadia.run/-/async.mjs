/** @returns {Promise<unknown>} */
export const sleep = (ms = 0) =>
  new Promise(resolve => setTimeout(resolve, ms));
