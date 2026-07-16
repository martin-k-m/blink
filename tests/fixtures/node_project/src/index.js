// Minimal entry point for the Node project fixture. References `express` so
// unused-dependency detection sees it as used; `eslint` is a devDependency and
// is intentionally not imported (dev deps are excluded from that check).
const express = require("express");

const app = express();

app.get("/", (_req, res) => {
  res.send("hello from the node-project fixture");
});

module.exports = app;
