---
title: Default Theme Page
theme: default
---

# Default Theme Override

This page uses the **default theme** even though the site is configured for Kanagawa.

## Purpose

This demonstrates that per-page theme overrides work correctly with the sherwood.toml configuration.

## Theme Hierarchy

1. **Site-wide setting** (`sherwood.toml`): `theme = "kanagawa"`
2. **Page-level override** (frontmatter): `theme: "default"`
3. **Variant selection** (frontmatter): `theme_variant: "dark"`

The page-level `theme` override takes precedence over the site-wide setting.