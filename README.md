# <img src="./stadia.run/-/squirrel-stadian.png" height="32" /> Stadians.dev

- Launcher: https://stadia.run
- Data: https://stadia.st/-/skus.json
- Code: https://github.com/stadians/stadians

The data update process isn't entirely automated, so we're probably missing recent changes.

Stadia store data, including product names, images, descriptions, and other metadata, may be copyrighted and/or trademarked by Google or other publishers. They are used here for non-commercial purposes assumed permitted as fair dealing.

The goal for stadia.run is to be an extremely fast way to access your games. In service of that, `index.html` needs to include everything required to do that directly inline: styles, scripts, and micro thumbnail placeholders for game covers. Supplemental scripts and high-resolution images can be loaded later, but they shouldn't block any of the main interactions.

However, I also want to keep this code as minimal as possible. So I'm not using any frameworks or build tools like React or WebPack. Instead, the page itself includes some very simple "dev tools" which you can activate by pressing <kbd>F12</kbd>. The idea is that the these tools will let you import new data to update the page and generate the new `index.html` in the browser.

For desktop browsers, I only care about supporting whatever Stadia supports, so that means we can take advantage of all of the new features that evergreen Chromium browsers have shipped.
