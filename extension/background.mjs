chrome.runtime.onMessageExternal.addListener(
  (message, _sender, sendResponse) => {
    onMessage(message).then(sendResponse);
    return true;
  }
);

const onMessage = async (message) => {
  if (message?.fetch) {
    const [path, ...rest] = message?.fetch;
    const url = `https://stadia.google.com/${path.replace(/^\//, "")}`;
    const response = await fetch(url, ...rest);
    const text = await response.text();
    return { text, status: response.status };
  } else if (message?.ping) {
    return "pong";
  } else {
    return {
      error: "what do you want?",
    };
  }
};
