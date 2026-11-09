//! Modo administrador permanente do SOLUDESK instalado (só para o usuário).
//!
//! Na instalação, o Windows pede UMA confirmação (UAC) a quem está no computador.
//! Com ela, o SOLUDESK cria uma tarefa no Agendador de Tarefas deste usuário com
//! "Executar com privilégios mais altos". Daí em diante o SOLUDESK instalado abre
//! sempre como administrador pela tarefa, sem nova confirmação, inclusive ao
//! entrar no Windows (se "Abrir ao entrar no Windows" estiver marcado). Nenhum
//! serviço, nenhuma conta SYSTEM e nada fora deste usuário. O portátil não usa.
//! Desinstalar remove a tarefa.

use super::{is_current_process_elevated, is_running_from_user_install, ResultType};
use hbb_common::{bail, log};
use std::os::windows::process::CommandExt;
use winreg::{enums::HKEY_CURRENT_USER, RegKey};

const OPTIONS_KEY: &str = "Software\\SOLUDESK";
const ADMIN_FLAG: &str = "AdminTask";
const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const RUN_VALUE: &str = "SOLUDESK";
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
/// Pede ao processo (já elevado pelo UAC) que crie a tarefa de administrador.
pub const SETUP_ADMIN_ARG: &str = "--soludesk-setup-admin";
/// Marca o processo aberto pela tarefa (evita reabrir pela tarefa de novo).
pub const FROM_TASK_ARG: &str = "--soludesk-from-task";

fn task_name() -> String {
    let user = std::env::var("USERNAME").unwrap_or_else(|_| "usuario".to_owned());
    format!("SOLUDESK\\Administrador - {}", user)
}

fn user_principal() -> String {
    let user = std::env::var("USERNAME").unwrap_or_default();
    match std::env::var("USERDOMAIN") {
        Ok(d) if !d.is_empty() => format!("{}\\{}", d, user),
        _ => user,
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn schtasks(args: &[&str]) -> bool {
    match std::process::Command::new("schtasks.exe")
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
    {
        Ok(out) => {
            if !out.status.success() {
                log::warn!(
                    "schtasks {:?} falhou: {}",
                    args.first(),
                    String::from_utf8_lossy(&out.stderr).trim()
                );
            }
            out.status.success()
        }
        Err(e) => {
            log::warn!("schtasks não executou: {}", e);
            false
        }
    }
}

/// A tarefa de administrador foi criada para este usuário.
pub fn admin_task_enabled() -> bool {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(OPTIONS_KEY)
        .and_then(|k| k.get_value::<String, _>(ADMIN_FLAG))
        .map(|v| v == "1")
        .unwrap_or(false)
}

fn startup_enabled() -> bool {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(OPTIONS_KEY)
        .and_then(|k| k.get_value::<String, _>("STARTUP"))
        .map(|v| v == "1")
        .unwrap_or(false)
}

/// Cria (ou recria) a tarefa. Só funciona no processo elevado do SOLUDESK
/// instalado deste mesmo usuário (a elevação veio do UAC aceito por ele).
pub fn create_admin_task() -> ResultType<()> {
    if !is_current_process_elevated() {
        bail!("o SOLUDESK não está rodando como administrador");
    }
    if !is_running_from_user_install() {
        // Ex.: conta comum que digitou a senha de outro administrador no UAC:
        // o processo elevado é de outra conta e não tem esta instalação.
        bail!("a instalação do SOLUDESK não é desta conta do Windows");
    }
    let exe = std::env::current_exe()?;
    let dir = exe.parent().map(|p| p.to_path_buf()).unwrap_or_default();
    let principal = xml_escape(&user_principal());
    let trigger = if startup_enabled() {
        format!(
            "<LogonTrigger><Enabled>true</Enabled><UserId>{}</UserId><Delay>PT5S</Delay></LogonTrigger>",
            principal
        )
    } else {
        String::new()
    };
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Description>Abre o SOLUDESK como administrador. Criada com a confirmação do usuário na instalação; removida ao desinstalar.</Description>
  </RegistrationInfo>
  <Triggers>{trigger}</Triggers>
  <Principals>
    <Principal id="Author">
      <UserId>{principal}</UserId>
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>HighestAvailable</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>Parallel</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <ExecutionTimeLimit>PT0S</ExecutionTimeLimit>
    <AllowHardTerminate>true</AllowHardTerminate>
    <Enabled>true</Enabled>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>{exe}</Command>
      <Arguments>{arg}</Arguments>
      <WorkingDirectory>{dir}</WorkingDirectory>
    </Exec>
  </Actions>
</Task>
"#,
        trigger = trigger,
        principal = principal,
        exe = xml_escape(&exe.to_string_lossy()),
        arg = FROM_TASK_ARG,
        dir = xml_escape(&dir.to_string_lossy()),
    );
    let tmp = std::env::temp_dir().join(format!("soludesk-tarefa-{}.xml", std::process::id()));
    let mut bytes = vec![0xFFu8, 0xFE];
    for u in xml.encode_utf16() {
        bytes.extend_from_slice(&u.to_le_bytes());
    }
    std::fs::write(&tmp, bytes)?;
    let name = task_name();
    let tmp_s = tmp.to_string_lossy().to_string();
    let ok = schtasks(&["/Create", "/TN", &name, "/XML", &tmp_s, "/F"]);
    std::fs::remove_file(&tmp).ok();
    if !ok {
        bail!("o Agendador de Tarefas recusou a criação da tarefa");
    }
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (opts, _) = hkcu.create_subkey(OPTIONS_KEY)?;
    opts.set_value(ADMIN_FLAG, &"1".to_owned())?;
    // A tarefa passa a abrir o SOLUDESK ao entrar no Windows (já como
    // administrador); a entrada comum de inicialização deixaria duas cópias.
    if let Ok(run) = hkcu.open_subkey_with_flags(RUN_KEY, winreg::enums::KEY_SET_VALUE) {
        run.delete_value(RUN_VALUE).ok();
    }
    log::info!("Tarefa de administrador criada: {}", name);
    Ok(())
}

fn main_window_exists() -> bool {
    use winapi::um::winuser::FindWindowW;
    let class: Vec<u16> = "FLUTTER_RUNNER_WIN32_WINDOW\0".encode_utf16().collect();
    let title: Vec<u16> = format!("{}\0", crate::get_app_name()).encode_utf16().collect();
    unsafe { !FindWindowW(class.as_ptr(), title.as_ptr()).is_null() }
}

/// Abertura comum (atalho, ícone) do SOLUDESK instalado com modo administrador:
/// reabre pela tarefa, já elevado. `true` = a tarefa foi iniciada e este
/// processo deve sair.
pub fn relaunch_via_admin_task() -> bool {
    if !admin_task_enabled()
        || is_current_process_elevated()
        || !is_running_from_user_install()
        || main_window_exists()
    {
        return false;
    }
    let name = task_name();
    let ok = schtasks(&["/Run", "/TN", &name]);
    if ok {
        log::info!("SOLUDESK reaberto como administrador pela tarefa");
    }
    ok
}

/// Remove a tarefa ao desinstalar. Se o Windows exigir, pede confirmação (UAC)
/// para o próprio Agendador de Tarefas apagar.
pub fn remove_admin_task() {
    if !admin_task_enabled() {
        return;
    }
    let name = task_name();
    if !schtasks(&["/Delete", "/TN", &name, "/F"]) {
        let args = format!("/Delete /TN \"{}\" /F", name);
        if let Err(e) = super::run_uac("schtasks.exe", &args) {
            log::warn!("Não foi possível remover a tarefa de administrador: {}", e);
        }
    }
}
