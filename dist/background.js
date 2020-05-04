"use strict";
chrome.pageAction.onClicked.addListener((tabs) => {
  console.log("clicked", tabs);
  tabs.executeScript({
    file: "plugin.js",
  });
});
//# sourceMappingURL=background.js.map
