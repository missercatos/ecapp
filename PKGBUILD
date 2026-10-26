# Maintainer: missercatos <missercatos@gmail.com>

pkgname=ecapp
pkgver=0.4.0
pkgrel=1
pkgdesc="Terminal Translation Tool"
arch=('x86_64')
url="https://github.com/missercatos/ecapp"
license=('MIT')
depends=('glibc')
source=("https://github.com/missercatos/ecapp/releases/download/v$pkgver/ecapp-v$pkgver-x86_64-linux.tar.gz")
sha256sums=('4477a86588f8b1d7f769bcdb0ff547dfa54209e7fb40a2fe0099513fca67653f')

package() {
  install -Dm755 ecapp "$pkgdir/usr/bin/ecapp"
}
