#!/usr/bin/env bash
# Build aur/PKGBUILD with makepkg in a throwaway Arch Linux container, check it with namcap and
# test the installed package. Runs as root and installs packages; makepkg runs as "builder".
#
# Usage: build.sh [verify|local|auto]
#   verify  Build the PKGBUILD exactly as committed, downloading the helper-v<_helper> archive from
#           GitHub. This is what the AUR gets; the tag must already be pushed.
#   local   Build the helper from the checked-out commit instead and recompute the checksums.
#   auto    (default) verify if helper-v<_helper> exists and the checked-out helper matches it,
#           otherwise local.
# PKGBUILD, .SRCINFO and the package are left in $AUR_WORK (default /home/builder/aur).
#
# Locally: docker run --rm -v "$PWD":/src:ro -w /src archlinux:base-devel bash aur/build.sh
set -euo pipefail

mode=${1:-auto}
repo=$(cd -- "$(dirname -- "$0")/.." && pwd -P)
work=${AUR_WORK:-/home/builder/aur}

# The package depends on steam, which is in multilib.
if ! grep -q '^\[multilib\]' /etc/pacman.conf; then
  printf '\n[multilib]\nInclude = /etc/pacman.d/mirrorlist\n' >> /etc/pacman.conf
fi
pacman -Syu --noconfirm --needed desktop-file-utils git namcap pacman-contrib > /dev/null
id builder > /dev/null 2>&1 || useradd --create-home builder
echo 'builder ALL=(ALL) NOPASSWD: ALL' > /etc/sudoers.d/builder
git config --global --add safe.directory "$repo"

helper=$(sed -n 's/^_helper=//p' "$repo/aur/PKGBUILD")
cargo_version=$(sed -n 's/^version = "\(.*\)"$/\1/p' "$repo/helper/Cargo.toml")
if [[ $helper != "$cargo_version" ]]; then
  printf '_helper=%s in aur/PKGBUILD does not match version %s in helper/Cargo.toml.\n' "$helper" "$cargo_version" >&2
  exit 1
fi
tag=helper-v$helper
if [[ $mode == auto ]]; then
  mode=local
  if git -C "$repo" rev-parse -q --verify "refs/tags/$tag" > /dev/null &&
    git -C "$repo" diff --quiet "$tag" HEAD -- helper LICENSE .gitattributes; then
    mode=verify
  fi
fi
printf 'Building in %s mode.\n' "$mode"

rm -rf -- "$work"
install -d -o builder "$work"
cp "$repo/aur/PKGBUILD" "$work/"
if [[ $mode == local ]]; then
  # makepkg uses a source file that is already present instead of downloading it.
  git -C "$repo" archive --format=tar.gz --prefix="AstralParty.Toys.Linux-$tag/" \
    -o "$work/astralparty-toys-helper-$helper.tar.gz" HEAD
fi
chown -R builder "$work"
cd "$work"
if [[ $mode == local ]]; then
  sudo -u builder updpkgsums
fi
sudo -u builder makepkg --printsrcinfo > .SRCINFO
sudo -u builder makepkg --syncdeps --noconfirm

package=$(sudo -u builder makepkg --packagelist | awk '!/-debug-/')
# Warnings about the Windows payload are expected; errors are not.
namcap PKGBUILD "$package" | tee namcap.log
if grep -q ' E: ' namcap.log; then
  echo 'namcap reported errors.' >&2
  exit 1
fi

pacman -U --noconfirm "$package" > /dev/null
lib=/opt/astralparty-toys
test -f "$lib/app/AstralParty.Toys.exe"
test -f "$lib/webview2/msedgewebview2.exe"
test -f /usr/share/icons/hicolor/128x128/apps/astralparty-toys.png
desktop-file-validate /usr/share/applications/astralparty-toys.desktop
sudo -u builder astralparty-toys --version
# Through the /usr/bin link, the launcher must find the packaged toolbox before it looks for Steam.
empty=$(mktemp -d)
chown builder "$empty"
if output=$(sudo -u builder env HOME="$empty" XDG_DATA_HOME="$empty/data" astralparty-toys 2>&1); then
  echo 'The launcher started without Steam.' >&2
  exit 1
fi
printf '%s\n' "$output"
grep -q '未找到原生 Steam' <<< "$output"
echo 'The installed package passed its checks.'
