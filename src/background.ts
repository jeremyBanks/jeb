declare const chrome: any;
const {
  runtime: { getURL },
  browserAction,
  tabs,
  windows,
} = chrome;

browserAction.onClicked.addListener(async (activeTab: any) => {
  const url = getURL("./.htm");

  console.log({ activeTab });

  if (activeTab.url === url) {
    return;
  }

  const matchingTabs: Array<any> = await new Promise((resolve) =>
    tabs.query(
      {
        url: getURL("*"),
      },
      resolve
    )
  );

  if (matchingTabs.length > 0) {
    tabs.update(matchingTabs[0].id, { active: true, highlighted: true });
    for (const [i, tab] of matchingTabs.sort((a, b) => a.id - b.id).entries()) {
      if (i === 0) {
        tabs.update(tab.id, { highlighted: true, active: true });
        windows.update(tab.windowId, { focused: true });
      } else {
        tabs.update(tab.id, { highlighted: true });
      }
    }
  } else {
    await new Promise((resolve) => tabs.create({ url }, resolve));
  }
});

export {};
