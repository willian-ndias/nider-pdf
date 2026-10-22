//! Help ▸ Check for updates (issue #28): ask for the latest release and offer its download page.
//!
//! The desktop app supplies how to ask ([`PdfCraftApp::update_source`]), so this crate has no
//! network code; without a source (the web build, tests) the command opens the releases page.
//! PdfCraft never downloads or installs anything itself: the user downloads the new version.
//! It asks only when the user does: there is no check at start (the owner's decision).

use std::sync::Arc;

use egui::{Align, Layout};

use crate::{PdfCraftApp, theme, widgets};

/// Where every PdfCraft release is listed.
pub const RELEASES_PAGE: &str = "https://nider.com.br/downloads";

/// The latest published release.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    /// Its version tag, such as `v0.2.0`.
    pub version: String,
    /// Its page on [`RELEASES_PAGE`], where the downloads are.
    pub url: String,
}

/// Asks for the latest release (blocking; it runs on its own thread).
pub type UpdateSource = Arc<dyn Fn() -> Result<Release, String> + Send + Sync>;

/// Whether release `latest` (a tag such as `v0.2.0`) is newer than version `current` (`0.1.1`).
/// Pre-release and build suffixes are ignored; a version that doesn't parse is never newer.
pub fn is_newer(latest: &str, current: &str) -> bool {
    matches!((parse(latest), parse(current)), (Some(l), Some(c)) if l > c)
}

fn parse(v: &str) -> Option<(u64, u64, u64)> {
    let v = v.trim().trim_start_matches(['v', 'V']);
    let core = v.split(['-', '+']).next()?;
    let mut parts = core.split('.');
    let mut next = |required: bool| match parts.next() {
        Some(p) => p.parse::<u64>().ok(),
        None if required => None,
        None => Some(0),
    };
    let version = (next(true)?, next(false)?, next(false)?);
    parts.next().is_none().then_some(version)
}

/// Where a check is.
#[derive(Default)]
pub(crate) enum Check {
    #[default]
    Idle,
    #[cfg(not(target_arch = "wasm32"))]
    Running(std::sync::mpsc::Receiver<Result<Release, String>>),
    Done(Result<Release, String>),
}

/// Nider: "Atualizar agora" baixa o instalador e o abre em modo "/atualizar" (ele espera o app
/// fechar, troca a versão instalada e reabre o Nider PDF), como no Nider Etapas.
#[cfg(not(target_arch = "wasm32"))]
mod auto {
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, Ordering};

    static BAIXANDO: AtomicBool = AtomicBool::new(false);
    static ERRO: Mutex<Option<String>> = Mutex::new(None);

    pub(crate) fn baixando() -> bool {
        BAIXANDO.load(Ordering::Relaxed)
    }

    pub(crate) fn erro() -> Option<String> {
        ERRO.lock().ok().and_then(|g| g.clone())
    }

    fn set_erro(msg: Option<String>) {
        if let Ok(mut g) = ERRO.lock() {
            *g = msg;
        }
    }

    /// Só o link direto do instalador do site Nider (não a página de downloads).
    pub(crate) fn e_instalador(url: &str) -> bool {
        url.starts_with("https://nider.com.br/api/apps/") && !url.contains(['"', '\\', ' '])
    }

    pub(crate) fn atualizar(ctx: egui::Context, versao: String, link: String) {
        BAIXANDO.store(true, Ordering::Relaxed);
        set_erro(None);
        std::thread::spawn(move || {
            let resultado = (|| -> Result<(), String> {
                let nome: String = versao.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '-').take(40).collect();
                let destino = std::env::temp_dir().join(format!("nider-pdf-setup-{nome}.exe"));
                let mut cmd = std::process::Command::new("curl");
                cmd.args(["-s", "-f", "-L", "-m", "900", "-o"]).arg(&destino).arg(&link);
                #[cfg(windows)]
                {
                    use std::os::windows::process::CommandExt;
                    cmd.creation_flags(0x0800_0000);
                }
                if !cmd.status().map(|s| s.success()).unwrap_or(false) {
                    return Err("Não consegui baixar a versão nova. Confira a internet e tente de novo.".into());
                }
                // Só um instalador do Windows (começa com "MZ") é aberto.
                let mut cabeca = [0u8; 2];
                let lido = std::fs::File::open(&destino).and_then(|mut f| std::io::Read::read_exact(&mut f, &mut cabeca));
                if lido.is_err() || &cabeca != b"MZ" {
                    let _ = std::fs::remove_file(&destino);
                    return Err("O arquivo baixado não é um instalador do Windows.".into());
                }
                std::process::Command::new(&destino).arg("/atualizar").spawn().map_err(|e| format!("Não consegui abrir o instalador: {e}"))?;
                Ok(())
            })();
            match resultado {
                // O instalador assume: fecha o app (alterações não salvas são perguntadas antes).
                Ok(()) => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                Err(e) => {
                    set_erro(Some(e));
                    BAIXANDO.store(false, Ordering::Relaxed);
                }
            }
            ctx.request_repaint();
        });
    }
}

#[derive(Default)]
pub(crate) struct Updates {
    pub(crate) check: Check,
    /// The Updates dialog is showing.
    pub(crate) open: bool,
    /// Nider: the silent check at start already ran.
    pub(crate) started: bool,
    /// Nider: the running check is the silent one; the dialog opens only if a newer version exists.
    pub(crate) silent: bool,
}

impl PdfCraftApp {
    /// Help ▸ Check for updates: ask for the latest release and show the outcome.
    pub fn check_for_updates(&mut self) {
        let Some(source) = self.update_source.clone() else {
            self.open_url(RELEASES_PAGE);
            return;
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.updates.open = true;
            if matches!(self.updates.check, Check::Running(_)) {
                return;
            }
            let (tx, rx) = std::sync::mpsc::channel();
            let ctx = self.ctx.clone();
            std::thread::spawn(move || {
                // The receiver may be gone (the app quit): nothing to report to then.
                let _ = tx.send(source());
                if let Some(ctx) = ctx {
                    ctx.request_repaint();
                }
            });
            self.updates.check = Check::Running(rx);
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = source;
            self.open_url(RELEASES_PAGE);
        }
    }

    /// Pick up a finished check (each frame).
    pub(crate) fn poll_updates(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Nider: uma checagem silenciosa ao abrir; o aviso só aparece se houver versão nova.
            if !self.updates.started {
                self.updates.started = true;
                if let Some(source) = self.update_source.clone() {
                    if matches!(self.updates.check, Check::Idle) {
                        let (tx, rx) = std::sync::mpsc::channel();
                        let ctx = self.ctx.clone();
                        std::thread::spawn(move || {
                            let _ = tx.send(source());
                            if let Some(ctx) = ctx {
                                ctx.request_repaint();
                            }
                        });
                        self.updates.check = Check::Running(rx);
                        self.updates.silent = true;
                    }
                }
            }
            if let Check::Running(rx) = &self.updates.check {
                let result = match rx.try_recv() {
                    Ok(r) => r,
                    Err(std::sync::mpsc::TryRecvError::Empty) => return,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => Err("the update check stopped unexpectedly".into()),
                };
                let newer = matches!(&result, Ok(r) if is_newer(&r.version, env!("CARGO_PKG_VERSION")));
                self.updates.check = Check::Done(result);
                if self.updates.silent {
                    self.updates.silent = false;
                    if newer {
                        self.updates.open = true;
                    }
                }
            }
        }
    }
}

/// The Updates dialog.
pub(crate) fn dialog(app: &mut PdfCraftApp, ctx: &egui::Context) {
    if !app.updates.open {
        return;
    }
    let t = theme::Tokens::get(ctx);
    let current = env!("CARGO_PKG_VERSION");
    let mut close = false;
    let mut download: Option<String> = None;
    let modal = crate::widgets::show_modal(ctx, "updates", |ui| {
        ui.set_width(420.0);
        ui.horizontal(|ui| {
            ui.add(crate::icons::image("cloud", 22.0, t.accent));
            ui.label(egui::RichText::new(tl!("Check for updates")).font(theme::semibold(16.0)));
        });
        ui.add_space(8.0);
        match &app.updates.check {
            Check::Idle => {
                ui.label(tl!("No check has run yet."));
            }
            #[cfg(not(target_arch = "wasm32"))]
            Check::Running(_) => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(tl!("Checking for a newer version…"));
                });
            }
            Check::Done(Ok(r)) if is_newer(&r.version, current) => {
                let version = r.version.trim_start_matches(['v', 'V']);
                ui.label(egui::RichText::new(crate::i18n::fmt(tl!("Nider PDF {v} is available."), &[("v", version)])).strong());
                ui.label(
                    egui::RichText::new(crate::i18n::fmt(
                        tl!("You have version {c}. Download the new version from its release page."),
                        &[("c", current)],
                    ))
                    .color(t.text_muted),
                );
                download = Some(r.url.clone());
            }
            Check::Done(Ok(_)) => {
                ui.label(crate::i18n::fmt(tl!("Nider PDF {c} is up to date."), &[("c", current)]));
            }
            Check::Done(Err(e)) => {
                ui.label(crate::i18n::fmt(tl!("Couldn't check for updates: {e}"), &[("e", &e.to_string())]));
                ui.label(
                    egui::RichText::new(crate::i18n::fmt(
                        tl!("You have version {c}. All releases are listed at {page}."),
                        &[("c", current), ("page", RELEASES_PAGE)],
                    ))
                    .color(t.text_muted),
                );
            }
        }
        ui.add_space(10.0);
        ui.label(
            egui::RichText::new(tl!("Asks GitHub for the latest release. Nothing is downloaded or installed automatically."))
                .color(t.text_muted)
                .small(),
        );
        ui.add_space(12.0);
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(e) = auto::erro() {
                ui.label(egui::RichText::new(e).color(t.accent_text));
                ui.add_space(6.0);
            }
            if auto::baixando() {
                ui.horizontal(|ui| {
                    seta_baixando(ui, t.accent);
                    ui.label("Baixando a versão nova… o Nider PDF vai fechar, atualizar e abrir de novo.");
                });
                return;
            }
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if let Some(url) = download.take() {
                // Nider: com o link direto do instalador, atualiza sozinho (sem navegador).
                #[cfg(not(target_arch = "wasm32"))]
                if auto::e_instalador(&url) {
                    if widgets::pill_button(ui, "Atualizar agora", true).clicked() {
                        let versao = match &app.updates.check {
                            Check::Done(Ok(r)) => r.version.clone(),
                            _ => String::new(),
                        };
                        auto::atualizar(ui.ctx().clone(), versao, url);
                    }
                    close = widgets::pill_button(ui, tl!("Later"), false).clicked();
                    return;
                }
                let get = widgets::pill_button(ui, tl!("Download"), true).clicked();
                let later = widgets::pill_button(ui, tl!("Later"), false).clicked();
                close = get || later;
                download = get.then_some(url);
            } else if widgets::pill_button(ui, tl!("Close"), true).clicked() {
                close = true;
            }
        });
    });
    if modal.should_close() {
        close = true;
        download = None;
    }
    if close {
        app.updates.open = false;
        if let Some(url) = download {
            app.open_url(&url);
        }
    }
}

/// Seta para baixo que desce e some, em ciclo, enquanto o instalador baixa.
fn seta_baixando(ui: &mut egui::Ui, cor: egui::Color32) {
    ui.ctx().request_repaint();
    let (rect, _) = ui.allocate_exact_size(egui::vec2(18.0, 22.0), egui::Sense::hover());
    let fase = (ui.input(|i| i.time) * 1.5).fract() as f32;
    let tinta = cor.gamma_multiply(1.0 - fase * 0.85);
    let cx = rect.center().x;
    let y = rect.top() + 3.0 + fase * (rect.height() - 12.0);
    let p = ui.painter();
    let traco = egui::Stroke::new(2.0, tinta);
    p.line_segment([egui::pos2(cx, y - 5.0), egui::pos2(cx, y + 3.0)], traco);
    p.line_segment([egui::pos2(cx - 4.5, y), egui::pos2(cx, y + 5.0)], traco);
    p.line_segment([egui::pos2(cx + 4.5, y), egui::pos2(cx, y + 5.0)], traco);
}
