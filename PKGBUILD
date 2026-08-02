# Maintainer: missercatos <missercatos@gmail.com>

pkgname=ecapp-bin
pkgver=0.1.2
pkgrel=1
pkgdesc="Terminal Translation Tool"
arch=('x86_64')
url="https://github.com/missercatos/ecapp"
license=('MIT')
depends=('glibc')
source=("https://github.com/missercatos/ecapp/releases/download/v$pkgver/ecapp-v$pkgver-x86_64-linux.tar.gz")
sha256sums=('75dbfc4886c72680b43fe25741fcb946239220dcd85f0f02ee486a343b30234a')

package() {
  install -Dm755 ecapp "$pkgdir/usr/bin/ecapp"
}