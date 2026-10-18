#!/usr/bin/env bash
# Sync the repository's PKGBUILD/.SRCINFO with the sha256 of the release
# artifact built by CI.
#
# The published tarball is rebuilt by GitHub Actions, so its bytes (and hash)
# differ from a local build. Without this step the checked-in PKGBUILD keeps a
# hash that no download will ever match.
#
# Idempotent: skips the commit when nothing changed, and never downgrades a
# branch that already carries a newer version.
set -euo pipefail

VERSION="${VERSION:?VERSION is required}"
SHA256="${SHA256:?SHA256 is required}"
BRANCH="${BRANCH:-master}"
REMOTE="${REMOTE:-origin}"

git config user.name "github-actions[bot]"
git config user.email "github-actions[bot]@users.noreply.github.com"

for attempt in 1 2 3; do
  git fetch "$REMOTE" "$BRANCH"
  git checkout -B "$BRANCH" "$REMOTE/$BRANCH"

  current=$(sed -n 's/^pkgver=//p' PKGBUILD)
  if [ "$(printf '%s\n%s\n' "$current" "$VERSION" | sort -V | tail -n1)" != "$VERSION" ]; then
    echo "$BRANCH already at ${current} (newer than ${VERSION}), skipping"
    exit 0
  fi

  sed -i "s/^pkgver=.*/pkgver=${VERSION}/" PKGBUILD
  sed -i "s/^sha256sums=('.*')/sha256sums=('${SHA256}')/" PKGBUILD
  sed -i -E "s/^([[:space:]]*)pkgver = .*/\1pkgver = ${VERSION}/" .SRCINFO
  sed -i -E "s/^([[:space:]]*)sha256sums = .*/\1sha256sums = ${SHA256}/" .SRCINFO
  sed -i "s|releases/download/v[^/]*/ecapp-v[^/]*-x86_64-linux.tar.gz|releases/download/v${VERSION}/ecapp-v${VERSION}-x86_64-linux.tar.gz|" .SRCINFO

  git add PKGBUILD .SRCINFO
  if git diff --cached --quiet; then
    echo "PKGBUILD already up to date for v${VERSION}"
    exit 0
  fi

  git commit -m "release v${VERSION}: 用 CI 产物哈希同步 PKGBUILD"
  if git push "$REMOTE" "$BRANCH"; then
    echo "PKGBUILD synced on ${BRANCH} for v${VERSION} (sha256=${SHA256})"
    exit 0
  fi

  echo "push failed (attempt ${attempt}), retrying..."
  sleep 3
done

echo "failed to sync PKGBUILD on ${BRANCH}" >&2
exit 1
