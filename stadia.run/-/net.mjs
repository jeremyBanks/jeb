const devApiHost = "//dev-api.stadia.st:57482";
const chromeExtensionId = "faklgfkhnojnmccmjiifiljdhfjnacpb";

/** Wraps a promise with a timeout. */
const withTimeout = (ms, promise, errorResolution) => {
  return Promise.race([
    new Promise((_, reject) => {
      setTimeout(() => {
        reject(new Error(`timed out after ${ms}ms`));
      }, ms);
    }),
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
 * Whether we are able to communicate with our Chrome extension in order to
 * fetch pages from the https://stadia.google.com host.
 */
export const canFetchStadiaHost = Promise.resolve(async () => {
  try {
    return "pong" === (await withTimeout(100, chromeCall("ping")));
  } catch {
    return false;
  }
});

/**
 * Whether we are actually able to fetch information from the Stadia Store,
 * meaning that we're authenticated and not getting a major error.
 */
export const canFetchStadiaStore = canFetchStadiaHost.then(
  async (predicate) => {
    if (!predicate) {
      return false;
    }

    // If we're not authenticated as a Stadia user, we will be redirected.
    const response = await fetchStadia("store");
    return response.redirected === false;
  }
);

/**
 * Whether we are able to communicate with a local dev server, which is required
 * to save updated data in the project directory.
 */
export const canFetchDevApi = Promise.resolve(async () => {
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
});

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
