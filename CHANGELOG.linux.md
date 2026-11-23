# 更新日志（Linux 版）

上游的更新日志见 [CHANGELOG.md](CHANGELOG.md)。

## 0.1.0（2026-11-23）

- 从上游 [PR #10](https://github.com/higashitaniyume/AstralParty.Toys/pull/10) 独立为本仓库，当前基于上游 v2.4.4。
- 原生概览：游戏目录、运行状态、加载器版本、磁盘模组数和最新日志；通过 Steam 启动游戏。
- 安装、修复、更新和卸载 CesiumLoader 3.8.3，只为所选游戏设置针对可执行文件的 Proton DLL 覆盖。
- 不再修改上游 WebUI：构建时注入 Linux 通信层和概览页。
- 应用标识改为 `io.github.particleg.astralparty-toys-linux`。
- AUR 包 `astralparty-toys-linux`：推送 `linux-v*` tag 后由 GitHub Actions 在 Arch Linux 中构建、测试并发布。
