# Maintainer: missercatos <missercatos@gmail.com>

pkgname=ecapp
pkgver=0.5.0
pkgrel=1
pkgdesc="Terminal Translation Tool"
arch=('x86_64')
url="https://github.com/missercatos/ecapp"
license=('MIT')
depends=('glibc')
source=("https://github.com/missercatos/ecapp/releases/download/v$pkgver/ecapp-v$pkgver-x86_64-linux.tar.gz")
sha256sums=('f4ac3e3648646a2b2202fa06c2b8a45204f9c6fa2949f18d1f7516bd3c21d58b')

package() {
  install -Dm755 ecapp "$pkgdir/usr/bin/ecapp"
}
