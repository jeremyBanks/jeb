declare const chrome: any;

chrome.browserAction.onClicked.addListener((tab: any) => {
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
