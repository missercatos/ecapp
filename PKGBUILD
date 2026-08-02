# Maintainer: missercatos <missercatos@gmail.com>

pkgname=ecapp-bin
pkgver=0.1.1
pkgrel=1
pkgdesc="Terminal Translation Tool"
arch=('x86_64')
url="https://github.com/missercatos/ecapp"
license=('MIT')
depends=('glibc')
source=("https://github.com/missercatos/ecapp/releases/download/v$pkgver/ecapp-v$pkgver-x86_64-linux.tar.gz")
sha256sums=('d7fc5deb4a1feeb30a90797903495f8c45c1a5ff20ced576e5331dc4d604e583')

package() {
  install -Dm755 ecapp "$pkgdir/usr/bin/ecapp"
}