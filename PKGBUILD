# Maintainer: missercatos <missercatos@gmail.com>

pkgname=ecapp
pkgver=0.3.2
pkgrel=1
pkgdesc="Terminal Translation Tool"
arch=('x86_64')
url="https://github.com/missercatos/ecapp"
license=('MIT')
depends=('glibc')
source=("https://github.com/missercatos/ecapp/releases/download/v$pkgver/ecapp-v$pkgver-x86_64-linux.tar.gz")
sha256sums=('f90bcd2f82bedb32a9fde6fdabea8f0817416a41406f15e2f37cb4a11fc0fa20')

package() {
  install -Dm755 ecapp "$pkgdir/usr/bin/ecapp"
}
