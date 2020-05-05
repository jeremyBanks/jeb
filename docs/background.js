"use strict";
const {
  runtime: { getURL },
  browserAction,
} = chrome;
browserAction.onClicked.addListener((_activeTab) => {
  open(getURL("./index.html"));
});
//# sourceMappingURL=background.js.map
