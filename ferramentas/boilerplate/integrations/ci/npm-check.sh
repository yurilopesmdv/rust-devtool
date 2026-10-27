#!/bin/sh
set -eu
npm ci --no-audit --no-fund
npm run check
