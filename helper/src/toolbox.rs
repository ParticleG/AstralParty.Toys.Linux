//! The official Windows toolbox, run through Steam's Proton in a prefix of its own.

use crate::steam::Steam;
use std::{
    env, fs, io,
    os::unix::{fs::symlink, process::CommandExt},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

const EXECUTABLE: &str = "AstralParty.Toys.exe";
/// Where the package installs the toolbox (`app/`) and WebView2 (`webview2/`). Not under /usr:
/// Proton runs inside the Steam Linux Runtime container, which replaces /usr.
const PACKAGE_DIR: &str = "/opt/astralparty-toys";
/// The game's data (replays, logs) that the toolbox reads, inside a Proton prefix.
const GAME_DATA: &str = "drive_c/users/steamuser/AppData/LocalLow/feimo";
/// Opens the toolbox's steam:// links with the native Steam client instead of Proton's stub.
const STEAM_LINKS: &str = r#"REGEDIT4

[HKEY_CURRENT_USER\Software\Classes\steam]
@="URL:Steam Protocol"
"URL Protocol"=""

[HKEY_CURRENT_USER\Software\Classes\steam\shell\open\command]
@="\"C:\\windows\\system32\\winebrowser.exe\" \"%1\""
"#;

/// `$XDG_DATA_HOME/astralparty-toys`: the toolbox prefix and this program's records.
pub fn data_dir() -> Result<PathBuf, String> {
    let absolute = |name| env::var_os(name).map(PathBuf::from).filter(|path| path.is_absolute());
    let base = absolute("XDG_DATA_HOME").or_else(|| absolute("HOME").map(|home| home.join(".local/share")))
        .ok_or("无法确定用户数据目录。")?;
    Ok(base.join("astralparty-toys"))
}

pub struct Toolbox {
    app: PathBuf,
    /// The fixed-version WebView2 runtime, as a Windows path.
    webview2: Option<String>,
    pub data: PathBuf,
}

impl Toolbox {
    /// The packaged toolbox and WebView2 runtime.
    pub fn installed(data: PathBuf) -> Result<Self, String> {
        Self::at(Path::new(PACKAGE_DIR), data)
    }

    fn at(base: &Path, data: PathBuf) -> Result<Self, String> {
        let app = base.join("app");
        if !app.join(EXECUTABLE).is_file() {
            return Err(format!("找不到官方工具箱 {}，请重新安装软件包。", app.join(EXECUTABLE).display()));
        }
        let webview2 = base.join("webview2");
        let webview2 = webview2.join("msedgewebview2.exe").is_file().then(|| windows_path(&webview2)).transpose()?;
        Ok(Self { app, webview2, data })
    }

    fn compat(&self) -> PathBuf {
        self.data.join("compatdata")
    }

    fn prefix(&self) -> PathBuf {
        self.compat().join("pfx")
    }

    /// Whether the prefix still has to be created.
    pub fn is_new(&self) -> bool {
        !self.prefix().join("user.reg").is_file()
    }

    /// Creates the prefix on first use and routes its steam:// links to the native client.
    pub fn prepare(&self, proton: &Proton) -> Result<(), String> {
        match fs::read(self.prefix().join("user.reg")) {
            Ok(registry) if links_open_native_steam(&registry) => return Ok(()),
            Ok(_) => (),
            Err(error) if error.kind() == io::ErrorKind::NotFound => (),
            Err(error) => return Err(format!("无法读取工具箱的 Proton 注册表：{error}")),
        }
        fs::create_dir_all(self.compat()).map_err(|error| format!("无法创建 {}：{error}", self.compat().display()))?;
        let file = self.data.join("steam-links.reg");
        fs::write(&file, STEAM_LINKS).map_err(|error| format!("无法写入 {}：{error}", file.display()))?;
        // `run` also creates the prefix when it does not exist yet.
        let status = proton.command(self).args(["reg", "import"]).arg(windows_path(&file)?)
            .stdin(Stdio::null()).stdout(Stdio::null()).status();
        let _ = fs::remove_file(&file);
        match status {
            Ok(status) if status.success() => Ok(()),
            Ok(status) => Err(format!("无法初始化工具箱的 Proton 运行环境（{status}）。")),
            Err(error) => Err(format!("无法启动 Proton：{error}")),
        }
    }

    /// Links the game's data into the toolbox prefix so replays and logs are visible.
    pub fn link_game_data(&self, game_prefix: &Path) -> Result<(), String> {
        let source = game_prefix.join(GAME_DATA);
        if !source.is_dir() {
            return Ok(()); // The game has not been started through Proton yet.
        }
        let link = self.prefix().join(GAME_DATA);
        match fs::symlink_metadata(&link) {
            Ok(metadata) if metadata.is_symlink() => {
                if fs::read_link(&link).is_ok_and(|target| target == source) {
                    return Ok(());
                }
                // Only this program creates the link; the game prefix has moved.
                fs::remove_file(&link).map_err(|error| format!("无法更新 {}：{error}", link.display()))?;
            }
            Ok(_) => return Err(format!("工具箱前缀中已有单独的游戏数据目录 {}，未关联游戏的回放和日志。", link.display())),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let parent = link.parent().expect("GAME_DATA has a parent");
                fs::create_dir_all(parent).map_err(|error| format!("无法创建 {}：{error}", parent.display()))?;
            }
            Err(error) => return Err(format!("无法读取 {}：{error}", link.display())),
        }
        symlink(&source, &link).map_err(|error| format!("无法关联游戏数据 {}：{error}", link.display()))
    }

    /// Replaces this process with the toolbox. Returns only on failure.
    pub fn exec(&self, proton: &Proton) -> String {
        let error = proton.command(self).arg(self.app.join(EXECUTABLE)).exec();
        format!("无法启动 Proton：{error}")
    }
}

/// Proton Experimental inside the Steam Linux Runtime it requires, as Steam runs it.
pub struct Proton {
    steam_root: PathBuf,
    proton: PathBuf,
    entry_point: PathBuf,
}

impl Proton {
    pub fn experimental(steam: &Steam) -> Result<Self, String> {
        let (proton, entry_point) = steam.proton_experimental()?;
        Ok(Self { steam_root: steam.root.clone(), proton, entry_point })
    }

    fn command(&self, toolbox: &Toolbox) -> Command {
        let mut command = Command::new(&self.entry_point);
        command.args(["--verb=run", "--"]).arg(self.proton.join("proton")).arg("run")
            .current_dir(&toolbox.app)
            .env("STEAM_COMPAT_CLIENT_INSTALL_PATH", &self.steam_root)
            .env("STEAM_COMPAT_DATA_PATH", toolbox.compat())
            .env("STEAM_COMPAT_INSTALL_PATH", &toolbox.app)
            .env("STEAM_COMPAT_APP_ID", "0")
            .env("SteamAppId", "0")
            .env("SteamGameId", "0");
        if env::var_os("WINEDEBUG").is_none() {
            command.env("WINEDEBUG", "-all");
        }
        // Wine's own Direct3D 9 for WPF: with DXVK the window turns black once it is tiled.
        let mut overrides = env::var_os("WINEDLLOVERRIDES").filter(|value| !value.is_empty())
            .map(|mut value| { value.push(";"); value }).unwrap_or_default();
        overrides.push("d3d9=b");
        command.env("WINEDLLOVERRIDES", overrides);
        if let Some(folder) = &toolbox.webview2 {
            command.env("WEBVIEW2_BROWSER_EXECUTABLE_FOLDER", folder);
        }
        command
    }
}

/// The path through Wine's `Z:` drive, which maps the host's root directory.
fn windows_path(path: &Path) -> Result<String, String> {
    let text = path.to_str().filter(|_| path.is_absolute())
        .ok_or_else(|| format!("路径必须是 UTF-8 绝对路径：{}", path.display()))?;
    Ok(format!("Z:{}", text.replace('/', "\\")))
}

fn links_open_native_steam(registry: &[u8]) -> bool {
    let section = br"[Software\\Classes\\steam\\shell\\open\\command]";
    let mut active = false;
    for line in registry.split(|&byte| byte == b'\n').map(<[u8]>::trim_ascii) {
        if line.starts_with(b"[") {
            active = line.len() >= section.len() && line[..section.len()].eq_ignore_ascii_case(section);
        } else if active && line.starts_with(b"@=") {
            return line.windows(15).any(|window| window.eq_ignore_ascii_case(b"winebrowser.exe"));
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::steam::test_support::Scratch;

    fn toolbox(temp: &Scratch) -> Toolbox {
        temp.write("lib/app/AstralParty.Toys.exe", b"");
        Toolbox::at(&temp.0.join("lib"), temp.0.join("data")).unwrap()
    }

    #[test]
    fn finds_the_packaged_toolbox_and_optional_runtime() {
        let temp = Scratch::new();
        assert!(Toolbox::at(&temp.0, temp.0.join("data")).is_err());
        assert_eq!(toolbox(&temp).webview2, None);
        temp.write("lib/webview2/msedgewebview2.exe", b"");
        let expected = format!("Z:{}", temp.0.join("lib/webview2").display().to_string().replace('/', "\\"));
        assert_eq!(toolbox(&temp).webview2, Some(expected));
        assert!(windows_path(Path::new("relative")).is_err());
    }

    #[test]
    fn recognizes_only_the_native_steam_link_handler() {
        let proton_default = br#"WINE REGISTRY Version 2
[Software\\Classes\\steam\\shell\\open\\command] 1779444637
@="\"C:\\Program Files (x86)\\Steam\\Steam.exe\" -- \"%1\""
"#;
        assert!(!links_open_native_steam(proton_default));
        let native = br#"WINE REGISTRY Version 2
[Software\\Classes\\steam\\shell\\open\\command] 1794799778
#time=1dd75762a0abece
@="\"C:\\windows\\system32\\winebrowser.exe\" \"%1\""
"#;
        assert!(links_open_native_steam(native));
        assert!(!links_open_native_steam(b"[Software\\\\Classes\\\\other]\n@=\"winebrowser.exe\"\n"));
    }

    #[test]
    fn links_game_data_without_replacing_a_real_directory() {
        let temp = Scratch::new();
        let toolbox = toolbox(&temp);
        let game = temp.0.join("game/pfx");
        // Nothing to link before the game has run.
        toolbox.link_game_data(&game).unwrap();
        assert!(fs::symlink_metadata(toolbox.prefix().join(GAME_DATA)).is_err());

        temp.write("game/pfx/drive_c/users/steamuser/AppData/LocalLow/feimo/AstralParty_CN/Player.log", b"");
        toolbox.link_game_data(&game).unwrap();
        toolbox.link_game_data(&game).unwrap();
        assert!(toolbox.prefix().join(GAME_DATA).join("AstralParty_CN/Player.log").is_file());

        // A moved game prefix replaces the link.
        temp.write("moved/pfx/drive_c/users/steamuser/AppData/LocalLow/feimo/AstralParty_INT/Player.log", b"");
        toolbox.link_game_data(&temp.0.join("moved/pfx")).unwrap();
        assert!(toolbox.prefix().join(GAME_DATA).join("AstralParty_INT").is_dir());

        let own = Scratch::new();
        let separate = self::toolbox(&own);
        own.write("data/compatdata/pfx/drive_c/users/steamuser/AppData/LocalLow/feimo/notes.txt", b"keep");
        assert!(separate.link_game_data(&game).is_err());
        assert!(separate.prefix().join(GAME_DATA).join("notes.txt").is_file());
    }
}
