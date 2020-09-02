const devApiHost = "//dev-api.stadia.st:57482";
const chromeExtensionId = "faklgfkhnojnmccmjiifiljdhfjnacpb";

/** Wraps a promise with a timeout. */
const withTimeout = (ms, promise, resolution) => {
  resolution ??= Promise.reject(new Error(`timed out after ${ms}ms`));
  return Promise.race([
    new Promise((resolve) => setTimeout(() => resolve(resolution), ms)),
    promise,
  ]);
};

/** Verifies that a fetch response has an successful status code. */
const checkStatus = (/** @type Response */ response) => {
  if (response.ok) {
    return response;
  } else {
    throw new Error(`${response.status} ${response.statusText}`);
  }
};

/** RPC call to the Stadians.dev Chrome extension. */
const chromeCall = async (methodName, ...args) => {
  return new Promise((resolve) =>
    chrome.runtime.sendMessage(
      chromeExtensionId,
      {
        [methodName]: args,
      },
      {},
      (responses) => {
        const response = responses[methodName];
        if (response.error) {
          resolve(Promise.reject(response.error));
        } else {
          resolve(Promise.resolve(response.result));
        }
      }
    )
  );
};

/**
 * Whether we are able to communicate with our Chrome extension, which is
 * required to load updated data from Stadia.
 */
export const canFetchStadia = async () => {
  try {
    return "pong" === (await withTimeout(100, chromeCall("ping")));
  } catch {
    return false;
  }
};

/**
 * Whether we are able to communicate with a local dev server, which is required
 * to save updated data in the project directory.
 */
export const canFetchDevApi = async () => {
  try {
    const response = await withTimeout(
      100,
      fetch(`${devApiHost}/skus.json`).then(checkStatus)
    );
    await response.json();
    return true;
  } catch {
    return false;
  }
};

/**
 * Fetch a path on the stadia.google.com domain, through our Chrome extension.
 */
export const fetchStadia = async (path, options = {}) => {
  const response = await withTimeout(
    16_000,
    chromeCall("fetchStadia", path, options)
  );
  return {
    ...response,
    async text() {
      return this._text;
    },
    async json() {
      return JSON.parse(this._text);
    },
  };
};

/**
 * Fetch a path on our local dev API server.
 */
export const fetchDevApi = async (path, options = {}) => {
  return fetch(`${devApiHost}/${path}`, options).then(checkStatus);
};
