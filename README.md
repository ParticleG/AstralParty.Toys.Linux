# AstralParty.Toys.Linux

在 Linux 上使用 [AstralParty.Toys](https://github.com/higashitaniyume/AstralParty.Toys)（吉星派对工具箱）。由 [ParticleG](https://github.com/ParticleG) 独立维护，经上游作者同意发布（见 [上游 PR #10](https://github.com/higashitaniyume/AstralParty.Toys/pull/10)），不是上游的官方发布。Linux 相关的问题请在本仓库提交，不要发到上游。

本仓库不修改官方工具箱：软件包直接使用上游发布的 Windows 便携版，由一个小启动器 `astralparty-toys` 通过 Steam 的 Proton 运行它。因此官方的全部功能（回放、模组、皮肤、市场、变速、组队大厅等）与上游同步可用。

> 0.1.0 及之前的原生概览版（Tauri）已停止维护，代码保留在 tag `linux-v0.1.0`。

## 安装

Arch Linux 用户从 AUR 安装 [`astralparty-toys-bin`](https://aur.archlinux.org/packages/astralparty-toys-bin)：

```sh
yay -S astralparty-toys-bin
```

需要启用 multilib 仓库，并安装原生 Steam（不支持 Flatpak/Snap 版）。使用前：

1. 在 Steam 库的“工具”中安装 **Proton Experimental**。
2. 在 Steam 中安装吉星派对（AppID `2622000`），用 Proton 正常启动一次后退出。

然后从应用菜单打开“吉星派对工具箱”，或运行 `astralparty-toys`。首次启动会创建工具箱自己的 Proton 运行环境，大约需要一分钟。

## 启动器做了什么

- 用 Steam 中的 Proton Experimental 及其所需的 Steam Linux Runtime 运行官方工具箱。工具箱使用单独的 Proton 前缀 `~/.local/share/astralparty-toys/compatdata`，不放进游戏的前缀，设置、回放等数据也保存在这里。
- 使用软件包内置的固定版本 WebView2 运行时，不在前缀中安装 WebView2，也不会出现微软的自动更新程序。
- 把游戏前缀中的游戏数据目录（`AppData/LocalLow/feimo`）关联到工具箱前缀，让工具箱读到回放和日志。
- 让工具箱里的 `steam://` 链接交给本机 Steam 处理，“启动游戏”会通过 Steam 正常启动。
- 游戏没有运行时，通过 `protontricks` 为游戏主程序（`AstralParty_CN.exe` / `AstralParty_INT.exe`）添加专用的 `version=native,builtin` DLL 覆盖，CesiumLoader 才能在 Proton 中加载。已有覆盖（包括你自己设置的）不会被修改；本程序只记录并撤销自己添加的那一项。不修改 Steam 启动选项，也不设置全局覆盖。

CesiumLoader 的安装、更新和卸载都在官方工具箱中完成，启动器不往游戏目录写文件。

## 已知限制

- 工具箱和游戏在不同的 Proton 前缀中，工具箱看不到游戏进程，可能把运行中的游戏当成未运行。**游戏运行时不要安装、更新或卸载加载器。**
- 软件包安装在只读的系统目录，工具箱的自更新会提示“没有写入权限”，请通过 AUR 更新。上游发布新版本后，本仓库会尽快更新软件包。
- 只自动识别 Steam 默认库中的游戏；游戏装在其他库时，在工具箱中手动添加游戏目录。
- 工具箱窗口运行在 XWayland 中，部分 Wayland 合成器的窗口截图工具可能截到黑屏，这不影响使用。
- “移动设备”“组队大厅”等需要访问其他设备或额外服务的功能，在 Proton 下是否可用尚未验证。

## 卸载

1. 在工具箱中卸载 CesiumLoader（如需要）。
2. 运行 `astralparty-toys --uninstall`，撤销本程序为游戏添加的 DLL 覆盖。
3. `sudo pacman -R astralparty-toys-bin`。
4. 确认不再需要工具箱的设置和回放后，删除 `~/.local/share/astralparty-toys`。不要删除游戏的 `compatdata/2622000` 或存档。

## 从手动安装迁移

如果之前按 [Proton 运行指南](https://github.com/higashitaniyume/AstralParty.Toys/blob/master/docs/linux-proton.md) 手动安装过，工具箱前缀同样在 `~/.local/share/astralparty-toys/compatdata`，启动器会直接沿用其中的设置和回放。删除旧的启动脚本 `~/.local/bin/astralparty-toys` 和菜单项 `~/.local/share/applications/astralparty-toys.desktop`，否则它们会覆盖软件包提供的同名文件；该目录下手动解压的旧版程序文件也可以删除。

## 维护

- `helper/`：启动器源码（Rust，无第三方依赖）。`cargo test` 运行单元测试。
- `aur/PKGBUILD`：AUR 包。`aur/build.sh` 在一次性的 Arch Linux 容器中构建、检查并安装测试软件包：`docker run --rm -v "$PWD":/src:ro -w /src archlinux:base-devel bash aur/build.sh`。
- [`Upstream release`](.github/workflows/upstream-release.yml) 每天检查上游最新发布，自动把 `pkgver` 和校验值更新到 `update/astralparty-toys-<版本>` 分支并构建测试，通过就开 PR，失败就开 issue。
- [`AUR`](.github/workflows/aur.yml) 在 PR 中构建测试；合并到 `main` 后把 `PKGBUILD` 和 `.SRCINFO` 发布到 AUR（需要仓库 secret `AUR_SSH_PRIVATE_KEY`）。

发布新版启动器：

1. 修改 `helper/Cargo.toml` 的 `version`，提交并推送 tag：`git tag helper-v<版本> && git push origin helper-v<版本>`。
2. 在 `aur/PKGBUILD` 中把 `_helper` 改为新版本、`pkgrel` 加一，并填入 tag 源码包的校验值：`curl -sL https://github.com/ParticleG/AstralParty.Toys.Linux/archive/refs/tags/helper-v<版本>.tar.gz | sha256sum`。源码包不含 `aur/` 和 `.github/`，所以这一步可以在打 tag 之后提交。推送到 `main` 后自动发布。

更新 WebView2 运行时：在 [微软下载页](https://developer.microsoft.com/microsoft-edge/webview2/) 取得新版 Fixed Version x64 包的地址，更新 `_webview2`、`_webview2_id` 和对应的校验值。

## 许可

启动器以 [GNU AGPL-3.0](LICENSE) 发布，Copyright © 2026 ParticleG。官方工具箱（AGPL-3.0，Copyright © 2026 Yume Higashitani）及其内置的 CesiumLoader 按各自的许可分发；内置的 Astral Party 游戏素材归原权利人所有。WebView2 运行时在构建软件包时从微软下载，适用微软的许可条款。详见 [helper/NOTICE](helper/NOTICE)。
