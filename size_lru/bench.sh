#!/bin/bash
set -e

cargo bench --bench bench --features all
node table.js
node svg.js
