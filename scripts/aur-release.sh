#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: $0 <version>"
  echo "Example: $0 0.2.0"
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
echo "  SHA256: $SHA256"

echo "=== Updating PKGBUILD ==="

sed -i "s/^pkgver=.*/pkgver=$VERSION/" PKGBUILD
sed -i "s/^sha256sums=('.*')/sha256sums=('$SHA256')/" PKGBUILD

echo "=== Git commit & tag & push (GitHub) ==="

git add PKGBUILD
git commit -m "release v${VERSION}"
git tag "$TAG"
git push "$REMOTE" HEAD
git push "$REMOTE" "$TAG"

echo "=== Syncing AUR ecapp-bin ==="

AUR_REMOTE="ssh://aur@aur.archlinux.org/ecapp-bin.git"
AUR_TMP=$(mktemp -d)

git clone "$AUR_REMOTE" "$AUR_TMP"
cp PKGBUILD "$AUR_TMP/"
cd "$AUR_TMP"
if command -v makepkg &>/dev/null; then
  makepkg --printsrcinfo > .SRCINFO
fi
git add PKGBUILD .SRCINFO
git commit -m "update to v${VERSION}"
git push

rm -rf "$AUR_TMP"

echo ""
echo "=== Done. ecapp-bin v${VERSION} released to GitHub and AUR. ==="
echo "  GitHub:  https://github.com/missercatos/ecapp/releases/tag/${TAG}"
echo "  AUR:     https://aur.archlinux.org/packages/ecapp-bin"
