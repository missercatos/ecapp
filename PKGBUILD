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
sha256sums=('a6bb2aca0b9508384dcb54052ed1cc30464623718ec1a378f61cdcd830c95d1c')

package() {
  install -Dm755 ecapp "$pkgdir/usr/bin/ecapp"
}