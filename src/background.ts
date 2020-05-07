declare const chrome: any;

chrome.browserAction.onClicked.addListener(async (activeTab: any) => {
  const url = chrome.runtime.getURL("./.htm");

  console.log({ activeTab });

  if (activeTab.url === url) {
    return;
  }

  const matchingTabs: Array<any> = await new Promise((resolve) =>
    chrome.tabs.query(
      {
        url: chrome.runtime.getURL("*"),
      },
      resolve
    )
  );

  if (matchingTabs.length > 0) {
    chrome.tabs.update(matchingTabs[0].id, { active: true, highlighted: true });
    for (const [i, tab] of matchingTabs.sort((a, b) => a.id - b.id).entries()) {
      if (i === 0) {
        chrome.tabs.update(tab.id, { highlighted: true, active: true });
        chrome.windows.update(tab.windowId, { focused: true });
      } else {
        chrome.tabs.update(tab.id, { highlighted: true });
      }
    }
  } else {
    await new Promise((resolve) => chrome.tabs.create({ url }, resolve));
  }
});

export {};
