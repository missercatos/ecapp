# Maintainer: missercatos <missercatos@gmail.com>

pkgname=ecapp
pkgver=0.3.0
pkgrel=1
pkgdesc="Terminal Translation Tool"
arch=('x86_64')
url="https://github.com/missercatos/ecapp"
license=('MIT')
depends=('glibc')
source=("https://github.com/missercatos/ecapp/releases/download/v$pkgver/ecapp-v$pkgver-x86_64-linux.tar.gz")
sha256sums=('38edd96a826c2436e6aba89309baa211e62b9594fa9ad95b81ff4a124cbcad44')

package() {
  install -Dm755 ecapp "$pkgdir/usr/bin/ecapp"
}
