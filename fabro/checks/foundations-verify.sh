#!/usr/bin/env bash
set -euo pipefail

test -f outputs/foundations/foundation-plan.md
test -f outputs/foundations/review.md
rg -q "^# Bootstrap Foundations Frontier and Revalidate Raspberry Truth$" outputs/foundations/foundation-plan.md
rg -q "^# Foundations Frontier Review$" outputs/foundations/review.md
