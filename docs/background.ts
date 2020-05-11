export const main = () => {
  browser.browserAction.onClicked.addListener(async (activeTab: any) => {
    const url = browser.runtime.getURL("./.htm");

    console.log({ activeTab });

    if (activeTab.url === url) {
      return;
    }

    const matchingTabs: Array<any> = await browser.tabs.query({
      url: browser.runtime.getURL("*"),
    });

    if (matchingTabs.length > 0) {
      for (const [i, tab] of matchingTabs
        .sort((a, b) => a.id - b.id)
        .entries()) {
        if (i === 0) {
          await browser.tabs.update(tab.id, {
            highlighted: true,
            active: true,
          });
          await browser.windows.update(tab.windowId, { focused: true });
        } else {
          browser.tabs.update(tab.id, { highlighted: true });
        }
      }
    } else {
      await browser.tabs.create({ url });
    }
  });
};
