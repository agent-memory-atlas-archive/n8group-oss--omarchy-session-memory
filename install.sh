#!/bin/sh
# Installs the osm engine for the Omarchy Session Memory plugin.
#
# This script ships *in the plugin*. `omarchy plugin add` git-clones this
# repository into ~/.config/omarchy/plugins/<id>/, so by the time you can read
# this you already have it, and there is nothing to download and verify before
# running it. That is the point.
#
# What it replaced was a script fetched from the v0.1.0 release. The Omarchy
# marketplace's security review of 58cc3f4 rejected that chain, correctly: the
# script and the binary digest it carried came from the same mutable release,
# so a replaced release — or a compromised publisher account — changes the
# executable and its claimed digest together, and the reviewed plugin contains
# no independent anchor. Moving the script into the reviewed tree moves the
# anchor with it.
#
# SHA256 below is therefore the trust root. It is part of the commit the
# marketplace validated. It is *not* fetched at run time, and it is not read
# from the `.sha256` published beside the binary: a checksum served by whoever
# served the file checks for corruption and for nothing else.
#
# The digest can only be known after the release workflow has built the
# binary, so it is always written here *after* a tag is published — see
# "Cutting a release" in README.md. Do not "fix" a mismatch by copying
# whatever the release currently serves into this file; a mismatch is either
# an unfinished release or the substitution this anchor exists to catch.
set -eu

TAG="v0.1.0"
SHA256="1bcf65c08f005d0b57a3d06d7abc3239b85feecb8d32ddf3ad87a816f5e469d6"
URL="https://github.com/n8group-oss/omarchy-session-memory/releases/download/${TAG}/osm-x86_64-unknown-linux-gnu"

arch="$(uname -m)"
if [ "$arch" != "x86_64" ]; then
  echo "this release ships x86_64-unknown-linux-gnu only; this machine is $arch" >&2
  echo "build from source instead: https://github.com/n8group-oss/omarchy-session-memory" >&2
  exit 1
fi

for dep in curl sha256sum; do
  command -v "$dep" >/dev/null 2>&1 || {
    echo "$dep is required and was not found on PATH" >&2
    exit 1
  }
done

tmp="$(mktemp -d)"
# Not exec'd below, precisely so this trap still runs: $tmp holds an
# executable copy of the engine, and leaving it in /tmp would be leaving a
# binary behind on a machine that asked for one install.
trap 'rm -rf "$tmp"' EXIT INT TERM

echo "downloading osm ${TAG}"
curl -fsSL -o "$tmp/osm" "$URL"

# The refusal is the feature. `sha256sum -c` exits non-zero on a mismatch and
# `set -e` stops here, before anything has been installed, with the standard
# "FAILED" line naming the file.
if ! echo "${SHA256}  $tmp/osm" | sha256sum -c -; then
  echo >&2
  echo "REFUSING TO INSTALL: the downloaded engine is not the binary this" >&2
  echo "plugin was reviewed against." >&2
  echo "  expected: ${SHA256}" >&2
  echo "  from:     ${URL}" >&2
  echo "Nothing has been installed. Report this rather than working around it." >&2
  exit 1
fi

chmod +x "$tmp/osm"

# Arguments are passed straight through to `osm install`, so
#   sh install.sh --dry-run
# prints every step and touches nothing, and `--prefix DIR` works the same way
# it does there. `osm install` copies the binary it is running, which is the
# verified one in $tmp.
"$tmp/osm" install "$@"
