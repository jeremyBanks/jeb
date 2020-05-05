declare const chrome: any;
const {
  runtime: { getURL },
  browserAction,
} = chrome;

browserAction.onClicked.addListener((_activeTab: unknown) => {
  open(getURL("./.htm"));
});
