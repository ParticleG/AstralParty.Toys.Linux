# 更新日志

## 启动器 0.2.0（AUR `astralparty-toys-bin` 2.5.2-1）

- 改为打包上游官方 Windows 便携版，通过 Steam 的 Proton Experimental 运行，提供官方的全部功能；不再合并或修改上游源码。
- 新的启动器 `astralparty-toys`：创建工具箱专用的 Proton 前缀，使用内置的固定版本 WebView2，关联游戏数据，把 `steam://` 链接交给本机 Steam，并为游戏添加可撤销的专用 `version` DLL 覆盖。
- AUR 包改名为 `astralparty-toys-bin`，由 GitHub Actions 跟踪上游发布、构建测试并发布。
- 原生概览版（0.1.0，AUR `astralparty-toys-linux`）停止维护。

## 0.1.0（2026-11-23）

- 基于上游 v2.4.4 的原生 Linux 概览（Tauri），可管理 CesiumLoader 3.8.3 并通过 Steam 启动游戏。代码保留在 tag `linux-v0.1.0`。
