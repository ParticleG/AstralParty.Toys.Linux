#!/usr/bin/env bash
# Build the Linux package and check it the way users get it: Rust tests, the CesiumLoader
# lifecycle against the packaged .NET host, and the packaged window against a disposable
# Steam library. The Windows project is also compiled, since this repository patches some of its
# C# files. Used by Linux CI and the upstream sync. Never touches a real game or Steam.
#
# Needs Rust, the .NET 8 SDK, WebKitGTK 4.1 development files, WebKitWebDriver, xvfb-run,
# desktop-file-validate, curl, jq and unzip.
set -euo pipefail

cd "$(dirname "$0")"
# The Windows tests cannot run here; building them still checks the patched C# files after a merge.
dotnet build ../tests/AstralParty.Toys.Tests -c Release -p:EnableWindowsTargeting=true --nologo
cargo test --release --locked
bash build-linux.sh
work=$(mktemp -d)
trap 'rm -rf -- "$work"' EXIT
tar -xzf target/AstralParty.Toys-linux-x64.tar.gz -C "$work"
bundle=$work/AstralParty.Toys-linux-x64
desktop-file-validate "$bundle"/share/applications/*.desktop
bash ../tests/test-linux-loader.sh "$bundle/managed/AstralParty.Toys.LinuxHost"
xvfb-run -a python3 tests/gui-smoke.py "$bundle/astralparty-toys-linux"
