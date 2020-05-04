"use strict";
chrome.browserAction.onClicked.addListener((tab) => {
  if (tab.url.startsWith("https://stadia.google.com/")) {
    chrome.tabs.executeScript({
      file: "plugin.js",
    });
    chrome.browserAction.setBadgeText({
      text: "🕷️",
      tabId: tab.id,
    });
  }
});
//# sourceMappingURL=background.js.map
