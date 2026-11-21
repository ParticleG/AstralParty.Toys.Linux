# AstralParty.Toys.Linux

[AstralParty.Toys](https://github.com/higashitaniyume/AstralParty.Toys)（吉星派对工具箱）的原生 Linux 版，**实验性**。由 [ParticleG](https://github.com/ParticleG) 独立维护，经上游作者同意单独发布（见 [上游 PR #10](https://github.com/higashitaniyume/AstralParty.Toys/pull/10)），不是上游的官方发布。Linux 版的问题请在本仓库提交，不要发到上游。

本仓库的根目录 `README.md` 是上游的原文，保持不变以便同步。

## 功能与范围

窗口使用系统 GTK 3 和 WebKitGTK 4.1（Tauri 2），界面复用上游的 HTML、CSS 和 JavaScript，业务逻辑复用上游的 C# 代码。**游戏本身仍由 Proton 运行**，工具箱不需要 Wine、WebView2 或 Windows 版 .NET。

- 查看游戏目录、运行状态、加载器版本、磁盘上启用的模组数和最新的 CesiumLoader 日志；
- 通过 Steam 启动游戏（`steam://rungameid/2622000`）；
- 安装、修复、更新和卸载 CesiumLoader 3.8.3。

回放、皮肤、市场、模组配置等功能只有 Windows 版有。需要时按上游的 [Proton 运行指南](../docs/linux-proton.md) 使用 Windows 版，两个工具箱可以装在同一台电脑上，但不要同时用它们修改游戏。

仅支持 x86-64 和原生 Steam。不支持 Flatpak/Snap 版 Steam、ARM、TapTap 和非 Steam 安装；不提供 macOS 和 Windows 版（Windows 请直接使用上游）。

## 构建与安装

Arch Linux 用户可以从 AUR 安装 [`astralparty-toys-linux`](https://aur.archlinux.org/packages/astralparty-toys-linux)（首个版本发布后可用），它在本机从源码构建：

```sh
yay -S astralparty-toys-linux
```

其他发行版按下面的步骤自行构建。

需要 Rust 1.90 或更高版本、.NET 8 SDK、C 工具链、`pkg-config`、GTK 3 和 WebKitGTK 4.1 开发文件，以及 `curl`、`jq`、`unzip`、`tar`。管理加载器还需要原生 `protontricks`；只查看概览和从 Steam 启动则不需要。

- Arch Linux：`base-devel rust dotnet-sdk-8.0 gtk3 webkit2gtk-4.1 jq unzip xdg-utils protontricks`（已有 rustup 工具链时可不装 `rust`）。
- Debian/Ubuntu：WebKit 开发包是 `libwebkit2gtk-4.1-dev`。

在仓库根目录执行：

```sh
bash linux/build-linux.sh
mkdir -p "$HOME/.local/opt"
tar -xzf linux/target/AstralParty.Toys-linux-x64.tar.gz -C "$HOME/.local/opt"
"$HOME/.local/opt/AstralParty.Toys-linux-x64/astralparty-toys-linux"
```

每次构建请解压到空目录。`managed/` 是自包含的 Linux .NET 组件，必须和可执行文件放在一起。界面仍然依赖系统的 GTK/WebKit 库；要在其他发行版使用，请在该发行版或 glibc 兼容的系统上构建。

构建时会下载固定版本的 [CesiumLoader 3.8.3](https://github.com/higashitaniyume/CesiumLoader/releases/tag/modloader-3.8.3)，并按 [`linux/loader-release.json`](../linux/loader-release.json) 校验 SHA-256 后内嵌。已下载过的 ZIP 可以通过环境变量 `CESIUM_LOADER_ARCHIVE`（绝对路径）复用。安装好的工具箱不会在后台下载新版加载器。

添加到应用菜单（需要 `~/.local/bin` 在桌面会话的 `PATH` 中）：

```sh
bundle="$HOME/.local/opt/AstralParty.Toys-linux-x64"
id=io.github.particleg.astralparty-toys-linux
mkdir -p "$HOME/.local/bin"
ln -sf "$bundle/astralparty-toys-linux" "$HOME/.local/bin/astralparty-toys-linux"
install -Dm644 "$bundle/share/applications/$id.desktop" "$HOME/.local/share/applications/$id.desktop"
install -Dm644 "$bundle/share/icons/hicolor/256x256/apps/$id.png" "$HOME/.local/share/icons/hicolor/256x256/apps/$id.png"
```

## 使用

1. 在原生 Steam 中安装吉星派对（AppID `2622000`），正常启动一次后退出。工具箱读取 Steam 登记的游戏库和游戏清单，库和游戏目录不必使用默认名称，但游戏必须安装在本机。
2. 选择检测到的游戏版本。下拉框决定**查看或修改哪个目录**；「从 Steam 启动」遵循 Steam 自己的启动选择，不会强制使用下拉框里的版本，也不会绕过 Steam。
3. 查看加载器版本和启用的模组数。这些只是磁盘上的设置，不代表加载成功。游戏启动后点「刷新状态」，查看最新日志的时间和 `ok` / `failed` 统计；没有日志就无法确认运行结果。
4. 安装、修复、更新或卸载加载器前，**先退出游戏，刷新状态，确认目录，再点确认**。支持的版本是 **3.8.3**，不一定是上游最新版。已安装更高版本时不会降级，也不会替换来历不明的 `version.dll`。如果无法确认游戏已退出，或 Steam 正在更新游戏，请等待，不要强行操作。

加载器装在所选的 Windows 版游戏可执行文件旁边。工具箱只通过原生 `protontricks` 为该游戏的 Proton 前缀设置针对这个可执行文件的 `version` DLL 覆盖，不设置全局覆盖，也不修改 Steam 启动选项。找不到前缀或 Proton 版本对不上时会停止操作：请在 Steam 的游戏属性里选择兼容工具，启动并退出一次后再刷新。不要把工具箱指向其他前缀，也不要在 Wine 运行时编辑 `user.reg`。

可以识别 3.8.1、3.8.2 和内置 3.8.3 未经修改的官方文件清单。未知版本或清单被修改过时只能查看，不要手动替换清单来强制升级。

更新会保留模组的启用状态和玩家设置。卸载只删除与已安装清单一致的加载器文件，修改过的设置、第三方模组和日志会保留；原本就有的或被改动过的 DLL 覆盖不会被删除。如果要撤销对游戏的修改，请先在工具箱里卸载加载器，再删除工具箱目录、菜单项、图标和启动链接。不要删除游戏的 Steam `compatdata/2622000` 或存档。

如果游戏停在官方资源更新提示，请先在游戏里完成更新再正常启动。日志以 `AstralParty.Runtime timed out` 结尾**不代表**模组加载成功。

## 已验证

- Arch Linux、原生 Steam、Proton Experimental、国服：用 2026-11-19 的构建（当时 WebUI 仍有 Linux 补丁）在安装好的窗口中点「从 Steam 启动」，游戏正常启动，新生成的 CesiumLoader 3.8.3 日志为 `23 ok · 0 failed · 0 not loaded`。
- 当前构建（上游 WebUI 不做修改）在同一环境中正确显示了游戏目录、加载器版本、模组数和上述日志，但没有再实际启动游戏。从点击「从 Steam 启动」到调用 `xdg-open steam://rungameid/2622000` 的过程由 CI 的界面检查覆盖。
- 安装、升级和卸载只在临时游戏目录和独立的 Proton 前缀中测试过。

## 与上游同步

本仓库保留上游的完整历史，只跟随上游发布的版本（tag），不跟随上游 `master`。

- [`Upstream sync`](workflows/upstream-sync.yml) 每天检查上游最新发布：把它合并到 `sync/upstream-<tag>` 分支并运行 Linux CI，通过就开 PR，合并冲突或 CI 失败就开 issue。不会自动合并到 `main`。上游改动了 `.github/workflows/` 的版本，Actions 的令牌无权推送，工作流会直接开 issue，需要在本地合并后推送。
- 手动同步：`git fetch upstream --tags && git merge <tag>`。
- Linux 版自己的文件：`linux/`、`src/AstralParty.Toys.LinuxHost/`、`tests/test-linux-loader.sh`、`.github/README.md`、`.github/workflows/linux-ci.yml`、`.github/workflows/upstream-sync.yml`、`.github/workflows/aur.yml` 和 `CHANGELOG.linux.md`。上游的 WebUI 不做修改，构建时由 `linux/build.rs` 注入 Linux 通信层和概览页。
- 唯一仍在修改的上游文件是 `src/AstralParty.Toys/` 下的 20 个 C# 文件：把不依赖 WPF 的只读逻辑拆成独立文件，供 Linux 版编译。这些修改只在本仓库维护，不提交上游；上游改到这些文件时，同步可能产生冲突，需要手动解决。
- 上游的 `release.yml` 在本仓库的 Actions 设置中已停用，避免打 tag 时发布 Windows 版。文件本身保留，以便和上游无冲突合并。

[`Linux CI`](workflows/linux-ci.yml) 和本地检查使用同一个脚本：`bash linux/ci.sh`。它交叉编译 Windows 项目（检查本仓库修改过的 C# 文件），运行 Rust 测试，打包，再用打包产物测试加载器的完整生命周期，并在临时 Steam 库上打开窗口检查概览和启动按钮，不会读取真实的游戏或 Steam，也不会真的启动游戏。

## 发布版本

版本号以 `linux/Cargo.toml` 为准，tag 格式为 `linux-v<版本>`，和上游的 `v*` tag 区分开。

1. 修改 `linux/Cargo.toml` 的 `version`，运行 `cargo update --workspace --manifest-path linux/Cargo.toml` 更新 `Cargo.lock`，并在 `CHANGELOG.linux.md` 中写好这一版的说明，然后提交并推送。
2. 推送 tag：`git tag linux-v<版本> && git push origin linux-v<版本>`。不要用 `git push --tags`，本地仓库里还有上游的 tag。
3. [`AUR`](workflows/aur.yml) 工作流会在 Arch Linux 容器里用 [`linux/aur/PKGBUILD`](../linux/aur/PKGBUILD) 构建，安装后测试加载器生命周期和窗口，全部通过后把 `PKGBUILD` 和 `.SRCINFO` 推送到 AUR。

只改打包、不改版本时，手动运行 AUR 工作流，填同一个 tag 并调高 `pkgrel`。不填 tag 手动运行时，只构建和测试当前提交，不发布。本地可以在一次性容器中运行同一个脚本：`docker run --rm -v "$PWD":/src:ro -w /src archlinux:base-devel bash linux/aur/build.sh`。

首次发布前需要一次性设置：

1. 注册 [AUR](https://aur.archlinux.org/) 账号。
2. 生成一对专用的无口令 SSH 密钥：`ssh-keygen -t ed25519 -N '' -C 'AstralParty.Toys.Linux AUR' -f aur_deploy`。
3. 把 `aur_deploy.pub` 的内容填到 AUR 账号的 *My Account → SSH Public Key*。
4. 把私钥存为本仓库的 Actions secret，然后删除本地私钥：`gh secret set AUR_SSH_PRIVATE_KEY -R ParticleG/AstralParty.Toys.Linux < aur_deploy && rm aur_deploy`。

## 许可

与上游相同，以 [GNU AGPL-3.0](../LICENSE) 发布。上游代码 Copyright © 2026 Yume Higashitani，Linux 版改动 Copyright © 2026 ParticleG。

CesiumLoader 采用 AGPL-3.0（SDK 为 MIT），发布包附带其 `CesiumLoader-NOTICE.txt` 和 `CesiumLoader-LICENSE.txt`。界面中的 Astral Party 游戏素材归原权利人所有，不在本项目的授权范围内。
