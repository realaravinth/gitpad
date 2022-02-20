<div align="center">
<h1> GitPad </h1>
<p>

**Self-Hosted GitHub Gists**

</p>

[![Build](https://github.com/realaravinth/gitpad/actions/workflows/linux.yml/badge.svg)](https://github.com/realaravinth/gitpad/actions/workflows/linux.yml)
[![dependency status](https://deps.rs/repo/github/realaravinth/gitpad/status.svg)](https://deps.rs/repo/github/realaravinth/gitpad)
[![codecov](https://codecov.io/gh/realaravinth/gitpad/branch/master/graph/badge.svg)](https://codecov.io/gh/realaravinth/gitpad)

</div>

## Features

-   [ ] Upload code snippets(&check; REST API)
-   [ ] Syntax Highlighting
-   [ ] Comments
-   [x] Versioning through Git
-   [ ] Fork gists
-   [x] Gist privacy: public, unlisted, private
-   [ ] Git clone via HTTP and SSH
-   [ ] Activity Pub implementation for publishing native gists and commenting
-   [ ] Gitea OAuth integration

## Why?

Gists are nice, while there are wonderful forges like
[Gitea](https://gitea.io), there isn't a libre pastebin implementation that
can rival GitHub Gists.

## Usage

1. All configuration is done through
   [./config/default.toml](./config/default.toml)(can be moved to
   `/etc/gitpad/config.toml`).
