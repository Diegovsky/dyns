# Maintainer: Diego Augusto <diego.augusto@protonmail.com>
pkgname=dyns-userv
pkgver=1.1.0
pkgrel=1
epoch=
pkgdesc="A daemon that periodically updates the Cloudflare DNS entries"
arch=('any')
url=""
license=(MIT)
groups=()
depends=(jq curl)
makedepends=()
checkdepends=()
optdepends=()
provides=()
conflicts=()
replaces=()
backup=()
options=()
changelog=
source=(dyns dyns.service)
noextract=()
validpgpkeys=()
sha256sums=('f655e6cbcdb5fe7d0c1685ac64f84e8a2bdb97b8ae22738ba7cb16d547bd51e2'
            'b147a862f8ab7d53a53c981ceed33083b6374a944759f786eded7308a21240ea')


build() {
    chmod +x dyns
}

package() {
    install -d $pkgdir/sbin/
    install -d $pkgdir/etc/systemd/systemd/
    install dyns $pkgdir/sbin/
    install dyns.service $pkgdir/etc/systemd/systemd/
}
