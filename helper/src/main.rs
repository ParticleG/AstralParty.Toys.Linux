//! `astralparty-toys`: runs the official AstralParty.Toys Windows build through Steam's Proton
//! and lets the game load CesiumLoader through an executable-specific DLL override.

mod steam;
mod toolbox;

use std::{
    env,
    io::{self, IsTerminal},
    process::{Command, ExitCode, Stdio},
};
use toolbox::{Proton, Toolbox};

const USAGE: &str = "用法：astralparty-toys [--uninstall | --version | --help]

不带参数时，通过 Steam 的 Proton Experimental 启动官方 AstralParty.Toys（吉星派对工具箱）。

  --uninstall  撤销本程序为游戏添加的 version DLL 覆盖，卸载软件包前运行。
               不删除工具箱数据，也不删除游戏目录中的 CesiumLoader。
  --version    显示版本
  --help       显示本帮助
";

fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let result = match args.iter().map(|arg| arg.to_str()).collect::<Vec<_>>().as_slice() {
        [] => launch().map_err(|error| (error, "AstralParty.Toys 无法启动")),
        [Some("--uninstall")] => uninstall().map_err(|error| (error, "AstralParty.Toys 卸载未完成")),
        [Some("--version" | "-V")] => {
            println!("astralparty-toys {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        [Some("--help" | "-h")] => {
            print!("{USAGE}");
            Ok(())
        }
        _ => {
            eprint!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err((error, title)) => {
            eprintln!("astralparty-toys: {error}");
            notify(title, &error);
            ExitCode::FAILURE
        }
    }
}

fn launch() -> Result<(), String> {
    let toolbox = Toolbox::installed(toolbox::data_dir()?)?;
    let steam = steam::locate()?;
    let proton = Proton::experimental(&steam)?;
    let games = steam::discover().unwrap_or_else(|error| {
        warn(&error);
        Vec::new()
    });
    if toolbox.is_new() {
        notify("正在准备 AstralParty.Toys", "首次启动需要创建 Proton 运行环境，可能需要一分钟。");
    }
    toolbox.prepare(&proton)?;
    // Both editions share the app's Proton prefix.
    if let Some(game) = games.first()
        && let Err(error) = game.prefix().and_then(|prefix| toolbox.link_game_data(&prefix))
    {
        warn(&error);
    }
    for game in &games {
        // Never touch the prefix of a running or possibly running game.
        if game.running() != Some(false) {
            continue;
        }
        match game.ensure_override(&toolbox.data) {
            Ok(true) => eprintln!("astralparty-toys: 已为 {} 添加 version DLL 覆盖。", game.game_dir.display()),
            Ok(false) => (),
            Err(error) => warn(&error),
        }
    }
    Err(toolbox.exec(&proton))
}

fn uninstall() -> Result<(), String> {
    let data = toolbox::data_dir()?;
    let mut failures = Vec::new();
    for game in steam::discover()? {
        match game.restore_override(&data) {
            Ok(None) => (),
            Ok(Some(notice)) => println!("{}：{notice}", game.game_dir.display()),
            Err(error) => failures.push(format!("{}：{error}", game.game_dir.display())),
        }
    }
    if !failures.is_empty() {
        return Err(failures.join("\n"));
    }
    println!("已撤销本程序添加的 version DLL 覆盖（如有）。");
    println!("工具箱的设置、回放和 WebView2 数据仍在 {}，确认不再需要后可手动删除。", data.display());
    Ok(())
}

fn warn(message: &str) {
    eprintln!("astralparty-toys: 警告：{message}");
}

/// A desktop notification when started from the application menu, where stderr is not seen.
fn notify(summary: &str, body: &str) {
    if io::stderr().is_terminal() {
        return;
    }
    let _ = Command::new("notify-send").args(["--app-name=AstralParty.Toys", "--icon=astralparty-toys", summary, body])
        .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
}
