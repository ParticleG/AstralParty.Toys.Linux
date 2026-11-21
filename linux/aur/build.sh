#!/usr/bin/env bash
# Build the AUR package with makepkg and test what it installs. Used by the AUR workflow; to run it
# locally, use a disposable container:
#   docker run --rm -v "$PWD":/src -w /src archlinux:base-devel bash linux/aur/build.sh
#
# Runs as root and installs packages, so only run it in a throwaway Arch Linux container.
# makepkg and the tests run as the unprivileged user "builder".
# Usage: build.sh [linux-v<version> [pkgrel]]
# With a tag, the PKGBUILD downloads GitHub's archive of that tag. Without one, the checked-out
# commit is packaged under the version in linux/Cargo.toml. PKGBUILD, .SRCINFO and the package
# are left in $AUR_WORK (default /home/builder/aur).
set -euo pipefail

tag=${1:-}
pkgrel=${2:-1}
repo=$(cd -- "$(dirname -- "$0")/../.." && pwd -P)
work=${AUR_WORK:-/home/builder/aur}
pkgname=astralparty-toys-linux

pacman -Syu --noconfirm --needed git jq namcap pacman-contrib python webkitgtk-6.0 xorg-server-xvfb > /dev/null
id builder > /dev/null 2>&1 || useradd --create-home builder
echo 'builder ALL=(ALL) NOPASSWD: ALL' > /etc/sudoers.d/builder
git config --global --add safe.directory "$repo"

version=$(awk -F'"' '/^version = /{print $2; exit}' "$repo/linux/Cargo.toml")
if [[ -n $tag && $tag != "linux-v$version" ]]; then
  printf 'Tag %s does not match version %s in linux/Cargo.toml.\n' "$tag" "$version" >&2
  exit 1
fi
loader=$(jq -er .version "$repo/linux/loader-release.json")

rm -rf -- "$work"
install -d -o builder "$work"
sed -e "s/^pkgver=.*/pkgver=$version/" -e "s/^pkgrel=.*/pkgrel=$pkgrel/" -e "s/^_loader=.*/_loader=$loader/" \
  "$repo/linux/aur/PKGBUILD" > "$work/PKGBUILD"
if [[ -z $tag ]]; then
  # makepkg uses a source file that is already present instead of downloading it.
  git -C "$repo" archive --format=tar.gz --prefix="AstralParty.Toys.Linux-linux-v$version/" \
    -o "$work/$pkgname-$version.tar.gz" HEAD
fi
chown -R builder "$work"
cd "$work"
sudo -u builder updpkgsums
sudo -u builder makepkg --printsrcinfo > .SRCINFO
sudo -u builder makepkg --syncdeps --noconfirm
namcap PKGBUILD "$pkgname-$version-$pkgrel-x86_64.pkg.tar.zst" || true

pacman -U --noconfirm "$pkgname-$version-$pkgrel-x86_64.pkg.tar.zst" > /dev/null
sudo -u builder bash "$repo/tests/test-linux-loader.sh" "/usr/lib/$pkgname/managed/AstralParty.Toys.LinuxHost"
# Containers cannot create the namespaces of WebKit's sandbox; the test only loads the local page.
sudo -u builder env WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS=1 \
  xvfb-run -a python3 "$repo/linux/tests/gui-smoke.py" "/usr/bin/$pkgname"
