//! Instalação do SOLUDESK só para o usuário atual.
//!
//! Substitui a instalação completa herdada do RustDesk no botão "Instalar":
//! sem administrador, sem script `.bat`, sem serviço do Windows, sem firewall,
//! sem HKLM e sem alterar políticas do sistema. O layout é o mesmo do
//! instalador NSIS (`%LOCALAPPDATA%\Programs\SOLUDESK`), então os dois convivem.

use super::{installer_shell::shortcut_bytes, ResultType, SOLUDESK_VERSION};
use hbb_common::{anyhow::anyhow, bail, log};
use std::{
    fs,
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
};
use windows::Win32::{
    System::Com,
    UI::Shell::{
        FOLDERID_Desktop, FOLDERID_LocalAppData, FOLDERID_Programs, SHGetKnownFolderPath,
        KF_FLAG_DEFAULT,
    },
};
use winreg::{enums::HKEY_CURRENT_USER, RegKey};

const UNINSTALL_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\SOLUDESK";
const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const RUN_VALUE: &str = "SOLUDESK";
const OPTIONS_KEY: &str = "Software\\SOLUDESK";
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

pub(super) fn known_folder(id: &windows::core::GUID) -> ResultType<PathBuf> {
    let value = unsafe { SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None) }?;
    let path = unsafe { value.to_string() };
    unsafe { Com::CoTaskMemFree(Some(value.0.cast())) };
    Ok(PathBuf::from(path?))
}

fn app_name() -> String {
    crate::get_app_name()
}

/// `%LOCALAPPDATA%\Programs\SOLUDESK`
pub fn user_install_dir() -> ResultType<PathBuf> {
    Ok(known_folder(&FOLDERID_LocalAppData)?
        .join("Programs")
        .join(app_name()))
}

fn start_menu_dir() -> ResultType<PathBuf> {
    Ok(known_folder(&FOLDERID_Programs)?.join(app_name()))
}

fn desktop_shortcut() -> ResultType<PathBuf> {
    Ok(known_folder(&FOLDERID_Desktop)?.join(format!("{}.lnk", app_name())))
}

fn same_dir(a: &Path, b: &Path) -> bool {
    let norm = |p: &Path| {
        p.to_string_lossy()
            .trim_end_matches('\\')
            .to_lowercase()
            .replace('/', "\\")
    };
    norm(a) == norm(b)
}

fn current_exe_dir() -> ResultType<(PathBuf, String)> {
    let exe = std::env::current_exe()?;
    let name = exe
        .file_name()
        .ok_or_else(|| anyhow!("Nome do executável inválido"))?
        .to_string_lossy()
        .to_string();
    let dir = exe
        .parent()
        .ok_or_else(|| anyhow!("Pasta do executável inválida"))?
        .to_path_buf();
    Ok((dir, name))
}

/// O processo atual está rodando da pasta de instalação por usuário.
pub fn is_running_from_user_install() -> bool {
    match (current_exe_dir(), user_install_dir()) {
        (Ok((dir, _)), Ok(target)) => same_dir(&dir, &target),
        _ => false,
    }
}

/// Existe uma instalação por usuário registrada para este usuário.
pub fn is_user_installed() -> bool {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(UNINSTALL_KEY)
        .is_ok()
}

fn copy_dir_all(src: &Path, dst: &Path) -> ResultType<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&from, &to)?;
        } else {
            fs::copy(&from, &to).map_err(|e| {
                anyhow!(
                    "Não foi possível copiar {}: {} (feche outras janelas do SOLUDESK e tente de novo)",
                    to.display(),
                    e
                )
            })?;
        }
    }
    Ok(())
}

fn dir_size_kb(dir: &Path) -> u64 {
    fn walk(p: &Path) -> u64 {
        let Ok(rd) = fs::read_dir(p) else { return 0 };
        rd.flatten()
            .map(|e| match e.file_type() {
                Ok(t) if t.is_dir() => walk(&e.path()),
                Ok(_) => e.metadata().map(|m| m.len()).unwrap_or(0),
                Err(_) => 0,
            })
            .sum()
    }
    walk(dir) / 1024
}

fn write_shortcut(path: &Path, target: &str, args: Option<&str>) -> ResultType<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let bytes = shortcut_bytes(target, args, Some(target))?;
    fs::write(path, bytes)?;
    Ok(())
}

/// Opções lembradas da última instalação por usuário, no formato de
/// `get_install_options` (JSON com STARTMENUSHORTCUTS, DESKTOPSHORTCUTS, STARTUP).
pub fn user_install_options() -> String {
    let key = RegKey::predef(HKEY_CURRENT_USER).open_subkey(OPTIONS_KEY);
    let get = |name: &str, default: &str| -> String {
        key.as_ref()
            .ok()
            .and_then(|k| k.get_value::<String, _>(name).ok())
            .unwrap_or_else(|| default.to_owned())
    };
    serde_json::json!({
        "STARTMENUSHORTCUTS": get("STARTMENUSHORTCUTS", "1"),
        "DESKTOPSHORTCUTS": get("DESKTOPSHORTCUTS", "1"),
        "STARTUP": get("STARTUP", "0"),
        "PRINTER": "0",
    })
    .to_string()
}

/// Instala para o usuário atual. `options` usa as mesmas palavras da página de
/// instalação: `startmenu`, `desktopicon`, `startup`.
pub fn user_install(options: &str) -> ResultType<PathBuf> {
    let has = |o: &str| options.split_whitespace().any(|x| x == o);
    let (src_dir, exe_name) = current_exe_dir()?;
    let dest = user_install_dir()?;
    if !same_dir(&src_dir, &dest) {
        if dest.exists() && !dest.join(&exe_name).exists() && fs::read_dir(&dest)?.next().is_some() {
            bail!(
                "A pasta {} já existe e não parece ser do SOLUDESK; nada foi alterado.",
                dest.display()
            );
        }
        copy_dir_all(&src_dir, &dest)?;
    }
    let exe = dest.join(&exe_name);
    if !exe.exists() {
        bail!("O executável não foi encontrado em {}", exe.display());
    }
    let exe_s = exe.to_string_lossy().to_string();
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    // Entrada em "Aplicativos instalados" do usuário (HKCU, sem administrador).
    let (key, _) = hkcu.create_subkey(UNINSTALL_KEY)?;
    key.set_value("DisplayName", &app_name())?;
    key.set_value("DisplayVersion", &SOLUDESK_VERSION.to_owned())?;
    key.set_value("Publisher", &app_name())?;
    key.set_value("InstallLocation", &dest.to_string_lossy().to_string())?;
    key.set_value("DisplayIcon", &exe_s)?;
    key.set_value("UninstallString", &format!("\"{}\" --uninstall", exe_s))?;
    key.set_value("NoModify", &1u32)?;
    key.set_value("NoRepair", &1u32)?;
    key.set_value("EstimatedSize", &(dir_size_kb(&dest).min(u32::MAX as u64) as u32))?;

    // Atalhos.
    let start_menu = start_menu_dir()?;
    if has("startmenu") {
        write_shortcut(&start_menu.join(format!("{}.lnk", app_name())), &exe_s, None)?;
    } else {
        fs::remove_dir_all(&start_menu).ok();
    }
    let desktop = desktop_shortcut()?;
    if has("desktopicon") {
        write_shortcut(&desktop, &exe_s, None)?;
    } else {
        fs::remove_file(&desktop).ok();
    }

    // Abrir com o Windows: só para este usuário (HKCU\...\Run), sem serviço.
    let (run, _) = hkcu.create_subkey(RUN_KEY)?;
    if has("startup") && !super::admin_task_enabled() {
        run.set_value(RUN_VALUE, &format!("\"{}\"", exe_s))?;
    } else if !has("startup") {
        run.delete_value(RUN_VALUE).ok();
    }

    let (opts, _) = hkcu.create_subkey(OPTIONS_KEY)?;
    let flag = |b: bool| if b { "1".to_owned() } else { "0".to_owned() };
    opts.set_value("InstallDir", &dest.to_string_lossy().to_string())?;
    opts.set_value("STARTMENUSHORTCUTS", &flag(has("startmenu")))?;
    opts.set_value("DESKTOPSHORTCUTS", &flag(has("desktopicon")))?;
    opts.set_value("STARTUP", &flag(has("startup")))?;

    log::info!("SOLUDESK instalado para o usuário em {}", dest.display());
    Ok(exe)
}

/// Remove a instalação por usuário. A pasta é apagada logo depois que este
/// processo fecha, porque um executável em uso não pode se apagar.
pub fn user_uninstall() -> ResultType<()> {
    // Antes de apagar as opções: é lá que fica a marca da tarefa de administrador.
    super::remove_admin_task();
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(run) = hkcu.open_subkey_with_flags(RUN_KEY, winreg::enums::KEY_SET_VALUE) {
        run.delete_value(RUN_VALUE).ok();
    }
    hkcu.delete_subkey_all(UNINSTALL_KEY).ok();
    hkcu.delete_subkey_all(OPTIONS_KEY).ok();
    if let Ok(dir) = start_menu_dir() {
        fs::remove_dir_all(dir).ok();
    }
    if let Ok(lnk) = desktop_shortcut() {
        fs::remove_file(lnk).ok();
    }
    let dest = user_install_dir()?;
    // Só apaga a pasta padrão e se ela contém o SOLUDESK.
    if dest.join(format!("{}.exe", app_name().to_lowercase())).exists()
        || dest.join(format!("{}.exe", app_name())).exists()
    {
        let script = format!(
            "ping 127.0.0.1 -n 3 > nul & rmdir /s /q \"{}\"",
            dest.to_string_lossy()
        );
        std::process::Command::new("cmd")
            .args(["/c", &script])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .ok();
    }
    log::info!("Instalação por usuário do SOLUDESK removida");
    Ok(())
}

/// Argumento usado pelo processo novo para esperar o antigo fechar.
pub const WAIT_PID_ARG: &str = "--soludesk-wait-pid=";

/// O processo atual roda como administrador (token elevado). Calculado uma vez.
pub fn is_current_process_elevated() -> bool {
    static ELEVATED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ELEVATED.get_or_init(|| super::is_elevated(None).unwrap_or(false))
}

/// Reabre o SOLUDESK como administrador. Quem decide é a pessoa no computador,
/// na confirmação normal do Windows (UAC); nada é feito sem esse clique.
/// `Ok(true)`: o Windows abriu o processo novo, que espera este fechar.
/// `Ok(false)`: a pessoa cancelou a confirmação.
/// `setup_admin`: no SOLUDESK instalado, também cria a tarefa que mantém o modo
/// administrador nas próximas aberturas.
pub fn restart_elevated(setup_admin: bool) -> ResultType<bool> {
    let exe = std::env::current_exe()?.to_string_lossy().to_string();
    let mut arg = format!("{}{}", WAIT_PID_ARG, std::process::id());
    if setup_admin {
        arg = format!("{} {}", super::SETUP_ADMIN_ARG, arg);
    }
    super::run_uac(&exe, &arg)
}

/// Espera o processo `pid` (a janela anterior do SOLUDESK) fechar, no máximo 15 s.
pub fn wait_for_previous_instance(pid: u32) {
    use winapi::um::{
        handleapi::CloseHandle, processthreadsapi::OpenProcess, synchapi::WaitForSingleObject,
        winnt::SYNCHRONIZE,
    };
    if pid == 0 || pid == std::process::id() {
        return;
    }
    unsafe {
        let handle = OpenProcess(SYNCHRONIZE, 0, pid);
        if handle.is_null() {
            return; // já fechou
        }
        WaitForSingleObject(handle, 15_000);
        CloseHandle(handle);
    }
    // Dá tempo para o Windows liberar a janela e o canal de IPC do processo antigo.
    std::thread::sleep(std::time::Duration::from_millis(300));
}

/// Argumento do ícone da bandeja ligado a uma janela do SOLUDESK sem serviço.
pub const TRAY_OWNER_ARG: &str = "--soludesk-owner=";

/// O processo `pid` ainda está rodando.
pub fn is_process_alive(pid: u32) -> bool {
    use winapi::um::{
        handleapi::CloseHandle, processthreadsapi::OpenProcess, synchapi::WaitForSingleObject,
        winnt::SYNCHRONIZE,
    };
    use winapi::shared::winerror::WAIT_TIMEOUT;
    if pid == 0 {
        return false;
    }
    unsafe {
        let handle = OpenProcess(SYNCHRONIZE, 0, pid);
        if handle.is_null() {
            return false;
        }
        let alive = WaitForSingleObject(handle, 0) == WAIT_TIMEOUT;
        CloseHandle(handle);
        alive
    }
}

/// Fecha o SOLUDESK `pid` (usado por "Sair" na bandeja): pede para fechar pelo
/// canal interno e, se não fechar em 3 s, encerra o processo.
pub fn quit_owner(pid: u32) {
    use hbb_common::tokio;
    #[tokio::main(flavor = "current_thread")]
    async fn send_close() {
        if let Ok(mut conn) = crate::ipc::connect(1000, "").await {
            conn.send(&crate::ipc::Data::Close).await.ok();
        }
    }
    send_close();
    for _ in 0..30 {
        if !is_process_alive(pid) {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    use winapi::um::{
        handleapi::CloseHandle,
        processthreadsapi::{OpenProcess, TerminateProcess},
        winnt::PROCESS_TERMINATE,
    };
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if !handle.is_null() {
            TerminateProcess(handle, 0);
            CloseHandle(handle);
        }
    }
    log::info!("SOLUDESK {} encerrado pela bandeja", pid);
}
