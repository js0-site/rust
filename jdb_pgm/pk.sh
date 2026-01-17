#!/bin/bash
set -e

cargo bench --bench main -F pk

bun table.js
bun svg.js
