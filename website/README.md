# Website

The Blink landing page — a single, self-contained `index.html` (no build
step, no external assets: all CSS, JS, and the animated context-graph hero
are inline, so it works offline and can be served from anywhere).

It presents the **shipped** product and its positioning as a developer
context engine. The forward-looking roadmap deliberately lives in
[`docs/roadmap.md`](../docs/roadmap.md) and is not surfaced here, to avoid
advertising unbuilt features. The source of truth for the site's copy is
[`docs/WEBSITE_DATA.md`](../docs/WEBSITE_DATA.md).

## Preview

Open `index.html` directly in a browser, or serve the directory:

```sh
# any static server works; e.g.
python -m http.server -d website 8000
```

## Deploy

It's a static page. To publish via GitHub Pages, point Pages at this
directory (or copy `index.html` to the Pages source). Brand: near-black
`#0b0b0c` ground, `#ff2d8d` pink accent, the 👁️ mark.
