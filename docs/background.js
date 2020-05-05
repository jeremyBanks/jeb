"use strict";
const { runtime: { getURL }, browserAction, } = chrome;
browserAction.onClicked.addListener((_activeTab) => {
    open(getURL("./.htm"));
});
//# sourceMappingURL=background.js.map