#!/bin/bash
set -e

cargo bench --bench comparison --features all
./table.js
./svg.js
