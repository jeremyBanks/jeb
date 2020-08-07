Some data is at <https://stadia.observer/skus.json>, but it's not complete or
updated.

<https://stadia.observer> can show you some of it, but only in certain browsers.
This can be installed in Firefox and Chrome as an extension, but isn't published
in their stores.

This uses `docs/` when it should use `dist/` because that's the only path
GitHub Pages will serve from.

This uses JSX for convenience but just basic DOM operations underneath; no React
or any other framework. The content is mostly-static.

With `nvm` (Node Version Manager) installed you should be able to run `./build`.
