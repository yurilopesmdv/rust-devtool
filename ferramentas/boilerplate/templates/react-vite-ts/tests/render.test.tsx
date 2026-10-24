import React from 'react';
import test from 'node:test';
import assert from 'node:assert/strict';
import { renderToStaticMarkup } from 'react-dom/server';
import { Home } from "../src/pages/Home.js";
test('página inicial apresenta nome e ação de consultar API', () => {
 const html = renderToStaticMarkup(<Home />);
 assert.match(html, /{{PROJECT_NAME}}/);
 assert.match(html, /Consultar API/);
});
