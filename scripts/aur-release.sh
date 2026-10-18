#!/usr/bin/env bash
# Build, tag and push a release.
#
# GitHub Actions (.github/workflows/release.yml) rebuilds the tarball, publishes
# the release, syncs the AUR packages (ecapp + ecapp-bin) and finally writes the
# CI artifact's real sha256 back into PKGBUILD/.SRCINFO on master.
#
# The local build below is only a sanity check. Its hash is NOT the hash of the
# published artifact: CI rebuilds the binary, so the bytes differ. The provisional
# hash committed here is replaced by the workflow once the release exists.
set -euo pipefail

usage() {
  echo "Usage: $0 <version>"
  echo "Example: $0 0.3.2"
  exit 1
}

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
  usage
fi

ARCH="x86_64-linux"
TARBALL="ecapp-v${VERSION}-${ARCH}.tar.gz"
TAG="v${VERSION}"
REMOTE="${REMOTE:-origin}"

echo "=== Building ecapp v${VERSION} ==="

cargo build --release --locked

echo "=== Packaging ==="

mkdir -p dist
cp target/release/ecapp dist/
tar czf "dist/$TARBALL" -C dist ecapp

SHA256=$(sha256sum "dist/$TARBALL" | cut -d' ' -f1)
echo "  local SHA256: $SHA256 (provisional, CI will publish a different hash)"

echo "=== Updating PKGBUILD (provisional hash) ==="

sed -i "s/^pkgver=.*/pkgver=$VERSION/" PKGBUILD
sed -i "s/^sha256sums=('.*')/sha256sums=('$SHA256')/" PKGBUILD

echo "=== Git commit & tag & push (GitHub) ==="

git add PKGBUILD
git commit -m "release v${VERSION}"
git tag "$TAG"
git push "$REMOTE" HEAD
git push "$REMOTE" "$TAG"

echo ""
echo "=== Tag ${TAG} pushed. Remaining work runs in GitHub Actions: ==="
echo "  - build + publish the release tarball"
echo "  - sync AUR packages ecapp and ecapp-bin"
echo "  - backfill PKGBUILD/.SRCINFO with the CI artifact hash on master"
echo ""
echo "  Progress: https://github.com/missercatos/ecapp/actions/workflows/release.yml"
