//! Atualização do SOLUDESK pelo próprio app (GitHub Releases).
//!
//! O app baixa `soludesk-update.json` e a assinatura `soludesk-update.json.sig`
//! da última release de `github.com/soluerp/soludesk`. A assinatura Ed25519 é
//! conferida com a chave pública embutida (`soludesk_update_pubkey.txt`) antes de
//! qualquer outra coisa; a chave privada fica só no PC de quem publica. Depois o
//! `.exe` baixado tem o SHA-256 conferido contra o manifesto assinado.
//! Nada usa administrador, serviço ou script de instalação.

use super::{
    is_running_from_user_install, user_install, user_install_options,
    wait_for_previous_instance, ResultType, SOLUDESK_VERSION,
};
use hbb_common::{anyhow::anyhow, bail, log, sodiumoxide::crypto::sign};
use serde_derive::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    os::windows::process::CommandExt,
    path::PathBuf,
    sync::{atomic::AtomicBool, atomic::Ordering, Mutex},
    time::Duration,
};

const REPO_RELEASES: &str = "https://github.com/soluerp/soludesk/releases";
const MANIFEST_URL: &str =
    "https://github.com/soluerp/soludesk/releases/latest/download/soludesk-update.json";
const SIGNATURE_URL: &str =
    "https://github.com/soluerp/soludesk/releases/latest/download/soludesk-update.json.sig";
const PUBLIC_KEY_B64: &str = include_str!("soludesk_update_pubkey.txt");
/// Argumento do `.exe` novo: espera o antigo fechar e instala por cima.
pub const UPDATE_INSTALL_ARG: &str = "--soludesk-update-install=";
const CHECK_EVERY: Duration = Duration::from_secs(6 * 60 * 60);
const MAX_DOWNLOAD: u64 = 200 * 1024 * 1024;
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, Clone, Deserialize)]
struct Manifest {
    version: String,
    url: String,
    sha256: String,
    #[serde(default)]
    size: u64,
}

#[derive(Debug, Clone, Default, Serialize)]
struct State {
    /// Versão nova disponível (vazio = nenhuma).
    version: String,
    /// "", "downloading" ou "error".
    status: String,
    error: String,
    /// true quando roda instalado só para o usuário (atualiza por cima);
    /// false no portátil (baixa o novo .exe para Downloads e abre).
    installed: bool,
}

lazy_static::lazy_static! {
    static ref STATE: Mutex<State> = Mutex::new(State::default());
    static ref AVAILABLE: Mutex<Option<Manifest>> = Mutex::new(None);
}
static CHECKER_STARTED: AtomicBool = AtomicBool::new(false);
static DOWNLOADING: AtomicBool = AtomicBool::new(false);

/// Estado para a tela (JSON).
pub fn update_state_json() -> String {
    let mut s = STATE.lock().unwrap().clone();
    s.installed = is_running_from_user_install();
    serde_json::to_string(&s).unwrap_or_default()
}

/// Começa a verificar atualizações (uma vez por processo da janela principal):
/// 15 s depois de abrir e depois a cada 6 horas.
pub fn start_update_checker() {
    if CHECKER_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_secs(15));
        loop {
            match check_for_update() {
                Ok(Some(m)) => {
                    log::info!("SOLUDESK {} disponível", m.version);
                    STATE.lock().unwrap().version = m.version.clone();
                    *AVAILABLE.lock().unwrap() = Some(m);
                }
                Ok(None) => log::info!("SOLUDESK atualizado ({})", SOLUDESK_VERSION),
                Err(e) => log::warn!("Verificação de atualização falhou: {}", e),
            }
            std::thread::sleep(CHECK_EVERY);
        }
    });
}

fn public_key() -> ResultType<sign::PublicKey> {
    let raw = crate::common::decode64(PUBLIC_KEY_B64.trim())
        .map_err(|e| anyhow!("chave pública de atualização inválida: {}", e))?;
    let raw: [u8; 32] = raw
        .try_into()
        .map_err(|_| anyhow!("chave pública de atualização inválida"))?;
    Ok(sign::PublicKey(raw))
}

fn http_get(url: &str, limit: u64) -> ResultType<Vec<u8>> {
    let client = crate::hbbs_http::create_http_client_with_url(url);
    let resp = client
        .get(url)
        .timeout(Duration::from_secs(600))
        .send()?
        .error_for_status()?;
    if let Some(len) = resp.content_length() {
        if len > limit {
            bail!("arquivo maior que o esperado ({} bytes)", len);
        }
    }
    let bytes = resp.bytes()?;
    if bytes.len() as u64 > limit {
        bail!("arquivo maior que o esperado");
    }
    Ok(bytes.to_vec())
}

fn check_for_update() -> ResultType<Option<Manifest>> {
    let manifest_bytes = http_get(MANIFEST_URL, 64 * 1024)?;
    let sig_text = http_get(SIGNATURE_URL, 4 * 1024)?;
    let sig_raw = crate::common::decode64(String::from_utf8_lossy(&sig_text).trim())
        .map_err(|e| anyhow!("assinatura inválida: {}", e))?;
    if sig_raw.len() != 64 {
        bail!("assinatura inválida");
    }
    // Formato combinado do libsodium: assinatura Ed25519 (64 bytes) + mensagem.
    let mut signed = sig_raw;
    signed.extend_from_slice(&manifest_bytes);
    if sign::verify(&signed, &public_key()?).is_err() {
        bail!("assinatura do manifesto de atualização não confere; ignorado");
    }
    let m: Manifest = serde_json::from_slice(&manifest_bytes)?;
    let prefix = format!("{}/download/", REPO_RELEASES);
    if !m.url.starts_with(&prefix) || !m.url.to_lowercase().ends_with(".exe") {
        bail!("endereço de download fora do repositório oficial: {}", m.url);
    }
    if m.sha256.len() != 64 || !m.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("SHA-256 inválido no manifesto");
    }
    if hbb_common::get_version_number(&m.version)
        > hbb_common::get_version_number(SOLUDESK_VERSION)
    {
        Ok(Some(m))
    } else {
        Ok(None)
    }
}

fn downloads_dir() -> ResultType<PathBuf> {
    use windows::Win32::UI::Shell::FOLDERID_Downloads;
    super::soludesk_user::known_folder(&FOLDERID_Downloads)
}

/// Baixa, confere e aplica a atualização disponível. Roda em segundo plano;
/// em caso de sucesso este processo fecha.
pub fn start_update_download() {
    if DOWNLOADING.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::spawn(|| {
        {
            let mut s = STATE.lock().unwrap();
            s.status = "downloading".to_owned();
            s.error.clear();
        }
        if let Err(e) = download_and_apply() {
            log::error!("Atualização falhou: {}", e);
            let mut s = STATE.lock().unwrap();
            s.status = "error".to_owned();
            s.error = e.to_string();
        }
        DOWNLOADING.store(false, Ordering::SeqCst);
    });
}

fn download_and_apply() -> ResultType<()> {
    // Confere de novo na hora: o manifesto pode ter mudado desde a última checagem.
    let m = match check_for_update()? {
        Some(m) => m,
        None => bail!("nenhuma versão nova encontrada"),
    };
    let limit = if m.size > 0 { m.size.min(MAX_DOWNLOAD) } else { MAX_DOWNLOAD };
    let data = http_get(&m.url, limit)?;
    let digest = hex::encode(Sha256::digest(&data));
    if !digest.eq_ignore_ascii_case(&m.sha256) {
        bail!("o arquivo baixado não confere com a versão publicada (SHA-256)");
    }
    let file_name = format!("SOLUDESK-{}.exe", m.version);
    let pid = std::process::id();
    if is_running_from_user_install() {
        let dir = std::env::temp_dir().join("SOLUDESK-atualizacao");
        std::fs::create_dir_all(&dir)?;
        let exe = dir.join(&file_name);
        std::fs::write(&exe, &data)?;
        log::info!("Instalando SOLUDESK {} de {}", m.version, exe.display());
        std::process::Command::new(&exe)
            .arg(format!("{}{}", UPDATE_INSTALL_ARG, pid))
            .spawn()?;
    } else {
        // Portátil: guarda o novo .exe em Downloads e abre depois que este fechar
        // (os dois usam a mesma pasta de trabalho do portátil).
        let exe = downloads_dir()?.join(&file_name);
        std::fs::write(&exe, &data)?;
        log::info!("SOLUDESK {} salvo em {}", m.version, exe.display());
        let script = format!(
            "ping 127.0.0.1 -n 4 > nul & start \"\" \"{}\"",
            exe.to_string_lossy()
        );
        std::process::Command::new("cmd")
            .args(["/c", &script])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()?;
    }
    std::process::exit(0);
}

fn saved_install_words() -> String {
    let v: serde_json::Value =
        serde_json::from_str(&user_install_options()).unwrap_or(serde_json::Value::Null);
    let on = |k: &str| v.get(k).and_then(|x| x.as_str()) == Some("1");
    let mut words = Vec::new();
    if on("STARTMENUSHORTCUTS") {
        words.push("startmenu");
    }
    if on("DESKTOPSHORTCUTS") {
        words.push("desktopicon");
    }
    if on("STARTUP") {
        words.push("startup");
    }
    words.join(" ")
}

/// Executado pelo `.exe` novo: espera a versão antiga fechar, instala por cima
/// (só para o usuário, com as mesmas opções) e abre a versão instalada.
pub fn finish_update_install(old_pid: u32) {
    wait_for_previous_instance(old_pid);
    let words = saved_install_words();
    let mut last_err = String::new();
    // O ícone da bandeja e janelas de conexão da versão antiga levam alguns
    // segundos para sair e soltar os arquivos.
    for _ in 0..20 {
        match user_install(&words) {
            Ok(exe) => {
                log::info!("SOLUDESK atualizado em {}", exe.display());
                if let Err(e) = std::process::Command::new(&exe).spawn() {
                    log::error!("Falha ao abrir a versão instalada: {}", e);
                }
                return;
            }
            Err(e) => {
                last_err = e.to_string();
                std::thread::sleep(Duration::from_secs(1));
            }
        }
    }
    super::message_box(&format!(
        "Não foi possível concluir a atualização do SOLUDESK.\n\n{}\n\nFeche o SOLUDESK (ícone da bandeja → Sair) e abra este arquivo de novo.",
        last_err
    ));
}
