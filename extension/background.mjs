chrome.runtime.onMessageExternal.addListener(
  (message, _sender, sendResponse) => {
    handleRequest(message).then(sendResponse);
    return true;
  }
);

const rpcs = {
  async ping() {
    return "pong";
  },

  async fetchStadia(path, ...rest) {
    const url = `https://stadia.google.com/${path}`;
    const response = await fetch(url, ...rest);
    const text = await response.text();
    return {
      ok: response.ok,
      statusText: response.statusText,
      status: response.status,
      redirected: response.redirected,
      url: response.url,
      _text: text,
    };
  },
};

const handleRequest = async (request) => {
  const result = {};
  for (const key of Object.keys(request)) {
    const args = request[key];
    console.debug("request:", key, "(", ...args, ")");
    let resolution;
    try {
      resolution = { result: await rpcs[key](...args) };
      console.debug("result:", resolution);
    } catch (error) {
      resolution = { error: error.toString() };
      console.debug("error:", resolution);
    }
    result[key] = resolution;
  }
  return result;
};
