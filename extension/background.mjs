chrome.runtime.onMessageExternal.addListener(
  (message, _sender, sendResponse) => {
    onMessage(message).then(sendResponse);
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

const onMessage = async (message) => {
  const result = {};
  for (const key of Object.keys(message)) {
    const args = message[key];
    try {
      result[key] = { result: await rpcs[key](...args) };
    } catch (error) {
      result[key] = { error: error.toString() };
    }
  }
};
