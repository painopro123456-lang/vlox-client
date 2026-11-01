mod auth;
mod library;

use eframe::egui::{self, Align, Color32, FontId, Layout, RichText, Rounding, Stroke, Vec2};
use std::sync::mpsc::{self, Receiver, Sender};

const BG: Color32 = Color32::from_rgb(10, 13, 20);
const PANEL: Color32 = Color32::from_rgb(18, 24, 36);
const BLUE: Color32 = Color32::from_rgb(76, 139, 255);
const MUTED: Color32 = Color32::from_rgb(145, 158, 180);
const WHITE: Color32 = Color32::from_rgb(238, 243, 252);

#[derive(Clone, Copy, PartialEq)]
enum Page { Home, Mods, Modpacks, Settings, About }

struct VloxApp {
    page: Page,
    status: String,
    account_name: Option<String>,
    device_code: Option<(String, String)>,
    busy: bool,
    tx: Sender<auth::AuthEvent>,
    rx: Receiver<auth::AuthEvent>,
    mods: Vec<String>,
    instances: Vec<String>,
    selected_instance: Option<String>,
    library_status: String,
    modrinth_query: String,
    modrinth_results: Vec<library::ModrinthPack>,
    curseforge_query: String,
    curseforge_results: Vec<library::CurseForgePack>,
}

impl Default for VloxApp {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        let account_name = auth::saved_account_name();
        Self {
            page: Page::Home,
            status: if account_name.is_some() {
                "Microsoft account connected. Ready to launch.".into()
            } else {
                "Sign in with Microsoft to get started.".into()
            },
            account_name,
            device_code: None,
            busy: false,
            tx,
            rx,
            mods: library::list_mods().unwrap_or_default(),
            instances: library::list_instances().unwrap_or_default(),
            selected_instance: None,
            library_status: String::new(),
            modrinth_query: String::new(),
            modrinth_results: Vec::new(),
            curseforge_query: String::new(),
            curseforge_results: Vec::new(),
        }
    }
}

impl VloxApp {
    fn card<R>(ui: &mut egui::Ui, f: impl FnOnce(&mut egui::Ui) -> R) -> R {
        egui::Frame::none()
            .fill(PANEL)
            .rounding(Rounding::same(15.0))
            .stroke(Stroke::new(1.0, Color32::from_rgb(37, 47, 65)))
            .inner_margin(egui::Margin::same(18.0))
            .show(ui, f).inner
    }

    fn nav(&mut self, ui: &mut egui::Ui, label: &str, page: Page) {
        let active = self.page == page;
        if ui.add_sized(
            [190.0, 42.0],
            egui::Button::new(RichText::new(label).size(14.0).color(if active { WHITE } else { MUTED }))
                .fill(if active { Color32::from_rgb(30, 49, 79) } else { Color32::TRANSPARENT })
                .rounding(9.0)
                .stroke(Stroke::NONE),
        ).clicked() {
            self.page = page;
        }
    }

    fn handle_events(&mut self) {
        for event in self.rx.try_iter() {
            match event {
                auth::AuthEvent::DeviceCode { code, url } => {
                    self.device_code = Some((code, url));
                    self.status = "Complete the Microsoft sign-in in your browser.".into();
                    self.busy = true;
                }
                auth::AuthEvent::SignedIn { name } => {
                    self.account_name = Some(name.clone());
                    self.device_code = None;
                    self.status = format!("Signed in as {name}. Minecraft 26.2 is ready.");
                    self.busy = false;
                }
                auth::AuthEvent::LaunchProgress(message) => {
                    self.status = message;
                    self.busy = true;
                }
                auth::AuthEvent::LaunchStarted => {
                    self.status = "Minecraft Java 26.2 launched with Fabric and VLOX.".into();
                    self.busy = false;
                }
                auth::AuthEvent::LaunchExited(message) => {
                    self.status = message;
                    self.busy = false;
                }
                auth::AuthEvent::Failed(message) => {
                    self.status = message;
                    self.busy = false;
                }
            }
        }
    }

    fn sign_in(&mut self) {
        self.device_code = None;
        self.busy = true;
        self.status = "Requesting Microsoft sign-in…".into();
        auth::start_sign_in(self.tx.clone());
    }

    fn launch(&mut self) {
        if self.account_name.is_none() {
            self.status = "Sign in with Microsoft before launching Minecraft.".into();
            return;
        }
        self.busy = true;
        self.status = "Starting Minecraft 26.2…".into();
        let instance_dir = self.selected_instance.as_deref().and_then(|name| library::instance_path(name).ok());
        auth::start_launch_in(self.tx.clone(), instance_dir);
    }
}

impl eframe::App for VloxApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_events();
        ctx.request_repaint_after(std::time::Duration::from_millis(150));
        ctx.set_visuals(egui::Visuals::dark());
        let mut style = (*ctx.style()).clone();
        style.text_styles.insert(egui::TextStyle::Body, FontId::proportional(14.0));
        style.visuals.window_fill = BG;
        style.visuals.panel_fill = BG;
        style.spacing.item_spacing = Vec2::new(10.0, 10.0);
        ctx.set_style(style);

        egui::TopBottomPanel::top("custom_titlebar")
            .exact_height(42.0)
            .frame(egui::Frame::none().fill(Color32::from_rgb(12, 16, 24)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(12.0);
                    ui.label(RichText::new("V").size(20.0).strong().color(BLUE));
                    ui.label(RichText::new("VLOX CLIENT").size(12.0).strong().color(WHITE));
                    let drag = ui.allocate_response(
                        Vec2::new((ui.available_width() - 132.0).max(0.0), 30.0),
                        egui::Sense::click_and_drag(),
                    );
                    if drag.drag_started() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }
                    if ui.add_sized([36.0, 28.0], egui::Button::new("—").fill(Color32::TRANSPARENT).stroke(Stroke::NONE).rounding(5.0)).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                    }
                    let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
                    if ui.add_sized([36.0, 28.0], egui::Button::new(if maximized { "❐" } else { "□" }).fill(Color32::TRANSPARENT).stroke(Stroke::NONE).rounding(5.0)).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
                    }
                    if ui.add_sized([36.0, 28.0], egui::Button::new("×").fill(Color32::from_rgb(150, 45, 55)).stroke(Stroke::NONE).rounding(5.0)).clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });

        egui::SidePanel::left("side")
            .exact_width(224.0)
            .frame(egui::Frame::none().fill(Color32::from_rgb(12, 16, 24)).inner_margin(egui::Margin::same(16.0)))
            .show(ctx, |ui| {
                ui.add_space(10.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("V").size(30.0).strong().color(BLUE));
                    ui.vertical(|ui| {
                        ui.label(RichText::new("VLOX").size(19.0).strong().color(WHITE));
                        ui.label(RichText::new("CLIENT").size(10.0).strong().color(MUTED));
                    });
                });
                ui.add_space(34.0);
                ui.label(RichText::new("WORKSPACE").size(10.0).strong().color(MUTED));
                ui.add_space(8.0);
                self.nav(ui, "⌂   Home", Page::Home);
                self.nav(ui, "▦   Mod Manager", Page::Mods);
                self.nav(ui, "▤   Modpack Manager", Page::Modpacks);
                self.nav(ui, "⚙   Settings", Page::Settings);
                self.nav(ui, "ⓘ   About VLOX", Page::About);
                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    ui.separator();
                    ui.label(RichText::new("VLOX CLIENT  •  MINECRAFT 26.2").size(9.0).color(MUTED));
                    ui.horizontal(|ui| {
                        ui.colored_label(Color32::from_rgb(87, 205, 139), "●");
                        ui.label(RichText::new(if self.account_name.is_some() { "Account connected" } else { "Not signed in" }).size(11.0).color(MUTED));
                    });
                });
            });

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG).inner_margin(egui::Margin::same(28.0)))
            .show(ctx, |ui| match self.page {
                Page::Home => {
                    ui.label(RichText::new("YOUR GAME. YOUR WAY.").size(10.0).strong().color(BLUE));
                    ui.add_space(7.0);
                    ui.heading(RichText::new("Welcome to VLOX.").size(30.0).strong().color(WHITE));
                    ui.label(RichText::new("A direct Minecraft Java 26.2 launcher with VLOX built in.").color(MUTED));
                    ui.add_space(22.0);

                    egui::Frame::none()
                        .fill(Color32::from_rgb(18, 31, 52))
                        .rounding(Rounding::same(18.0))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(38, 66, 105)))
                        .inner_margin(egui::Margin::same(24.0))
                        .show(ui, |ui| {
                            ui.set_min_height(175.0);
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(RichText::new("MINECRAFT JAVA 26.2").size(10.0).strong().color(BLUE));
                                    ui.add_space(8.0);
                                    ui.label(RichText::new("Launch straight\ninto your world.").size(27.0).strong().color(WHITE));
                                    ui.add_space(14.0);
                                    ui.add_enabled_ui(!self.busy, |ui| {
                                        if ui.add_sized([190.0, 42.0], egui::Button::new(RichText::new("▶  Launch Minecraft").strong()).fill(BLUE).rounding(9.0)).clicked() {
                                            self.launch();
                                        }
                                    });
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(RichText::new("V").size(116.0).strong().color(Color32::from_rgb(48, 83, 137)));
                                });
                            });
                        });

                    ui.add_space(18.0);
                    ui.columns(2, |columns| {
                        Self::card(&mut columns[0], |ui| {
                            ui.label(RichText::new("MICROSOFT ACCOUNT").size(10.0).strong().color(MUTED));
                            if let Some(name) = self.account_name.as_deref() {
                                ui.label(RichText::new(name).size(19.0).strong().color(WHITE));
                                ui.label(RichText::new("Minecraft profile authenticated").color(MUTED));
                                ui.add_space(8.0);
                                if ui.button("Sign out").clicked() {
                                    match auth::sign_out() {
                                        Ok(()) => {
                                            self.account_name = None;
                                            self.status = "Signed out. Your saved Microsoft session was removed.".into();
                                        }
                                        Err(error) => self.status = error,
                                    }
                                }
                            } else {
                                ui.label(RichText::new("Not connected").size(19.0).strong().color(WHITE));
                                ui.label(RichText::new("Use your Microsoft account that owns Minecraft Java.").color(MUTED));
                                ui.add_space(8.0);
                                ui.add_enabled_ui(!self.busy, |ui| {
                                    if ui.button("Sign in with Microsoft").clicked() {
                                        self.sign_in();
                                    }
                                });
                            }
                        });
                        Self::card(&mut columns[1], |ui| {
                            ui.label(RichText::new("GAME PROFILE").size(10.0).strong().color(MUTED));
                            ui.label(RichText::new("Minecraft Java 26.2").size(19.0).strong().color(WHITE));
                            ui.label(RichText::new("Fabric Loader • VLOX Client • Fabric API").color(MUTED));
                            ui.add_space(8.0);
                            ui.label(RichText::new("Java 25 runtime managed by launcher engine").size(12.0).color(MUTED));
                        });
                    });

                    if let Some((code, url)) = self.device_code.as_ref() {
                        ui.add_space(14.0);
                        Self::card(ui, |ui| {
                            ui.label(RichText::new("MICROSOFT DEVICE SIGN-IN").size(10.0).strong().color(BLUE));
                            ui.label("Enter this code in the browser window:");
                            ui.label(RichText::new(code).size(28.0).strong().color(WHITE).monospace());
                            ui.hyperlink_to("Open Microsoft device sign-in", url);
                        });
                    }

                    ui.add_space(14.0);
                    Self::card(ui, |ui| {
                        ui.label(RichText::new("LAUNCHER STATUS").size(10.0).strong().color(MUTED));
                        ui.label(RichText::new(&self.status).color(WHITE));
                        if self.busy {
                            ui.add(egui::Spinner::new().color(BLUE));
                        }
                    });
                }
                Page::Mods => {
                    ui.heading(RichText::new("Mod Manager").size(28.0).strong().color(WHITE));
                    ui.label(RichText::new("Manage Fabric .jar mods installed in your default Minecraft 26.2 profile.").color(MUTED));
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        if ui.button("＋ Add .jar mod").clicked() {
                            match library::add_mod() {
                                Ok(message) => self.library_status = message,
                                Err(error) => self.library_status = error,
                            }
                            self.mods = library::list_mods().unwrap_or_default();
                        }
                        if ui.button("Open mods folder").clicked() {
                            if let Err(error) = library::open_mods_folder() { self.library_status = error; }
                        }
                        if ui.button("Refresh").clicked() { self.mods = library::list_mods().unwrap_or_default(); }
                    });
                    if !self.library_status.is_empty() {
                        ui.add_space(8.0);
                        ui.label(RichText::new(&self.library_status).color(BLUE));
                    }
                    ui.add_space(12.0);
                    Self::card(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("INSTALLED MODS  ·  {}", self.mods.len())).strong().color(WHITE));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(RichText::new("Fabric / Minecraft 26.2").size(11.0).color(MUTED));
                            });
                        });
                        ui.separator();
                        if self.mods.is_empty() {
                            ui.label(RichText::new("No .jar mods found yet. Add a mod or import a modpack.").color(MUTED));
                        }
                        for name in self.mods.clone() {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&name).color(WHITE));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui.small_button("Remove").clicked() {
                                        match library::remove_mod(&name) {
                                            Ok(()) => self.library_status = format!("Removed {name}."),
                                            Err(error) => self.library_status = error,
                                        }
                                        self.mods = library::list_mods().unwrap_or_default();
                                    }
                                });
                            });
                        }
                    });
                    ui.add_space(10.0);
                    ui.label(RichText::new("Changes to mods take effect the next time Minecraft starts. Only install mods you trust and that support Fabric 26.2.").size(11.0).color(MUTED));
                }
                Page::Modpacks => {
                    ui.heading(RichText::new("Modpack Manager").size(28.0).strong().color(WHITE));
                    ui.label(RichText::new("Browse Modrinth packs or import packs exported from Modrinth and CurseForge. Each pack gets an isolated game directory.").color(MUTED));
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        if ui.button("＋ Import Modrinth .mrpack").clicked() {
                            self.library_status = match library::import_modpack() {
                                Ok(message) => message,
                                Err(error) => error,
                            };
                            self.instances = library::list_instances().unwrap_or_default();
                        }
                        if ui.button("＋ Import CurseForge ZIP").clicked() {
                            self.library_status = match library::import_curseforge_modpack() {
                                Ok(message) => message,
                                Err(error) => error,
                            };
                            self.instances = library::list_instances().unwrap_or_default();
                        }
                        if ui.button("Refresh").clicked() { self.instances = library::list_instances().unwrap_or_default(); }
                    });
                    ui.add_space(12.0);
                    Self::card(ui, |ui| {
                        ui.label(RichText::new("MODRINTH DISCOVERY").strong().color(WHITE));
                        ui.horizontal(|ui| {
                            ui.add_sized([360.0, 30.0], egui::TextEdit::singleline(&mut self.modrinth_query).hint_text("Search modpacks for Minecraft 26.2"));
                            if ui.button("Search").clicked() {
                                self.library_status = "Searching Modrinth…".to_owned();
                                match library::search_modrinth_modpacks(&self.modrinth_query) {
                                    Ok(results) => {
                                        self.modrinth_results = results;
                                        self.library_status = format!("Found {} Modrinth modpacks for Fabric / Minecraft 26.2.", self.modrinth_results.len());
                                    }
                                    Err(error) => self.library_status = error,
                                }
                            }
                        });
                        for pack in self.modrinth_results.clone() {
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(&pack.title).strong().color(WHITE));
                                    ui.label(RichText::new(format!("{} downloads · {}", pack.downloads, pack.description)).size(11.0).color(MUTED));
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui.button("Install").clicked() {
                                        self.library_status = match library::install_modrinth_pack(&pack.slug) {
                                            Ok(message) => message,
                                            Err(error) => error,
                                        };
                                        self.instances = library::list_instances().unwrap_or_default();
                                    }
                                });
                            });
                        }
                    });
                    ui.add_space(10.0);
                    Self::card(ui, |ui| {
                        ui.label(RichText::new("CURSEFORGE DISCOVERY").strong().color(WHITE));
                        ui.label(RichText::new("Search and install CurseForge modpacks for Minecraft 26.2 / Fabric. VLOX connects securely through its API service.").size(11.0).color(MUTED));
                        ui.horizontal(|ui| {
                            ui.add_sized([360.0, 30.0], egui::TextEdit::singleline(&mut self.curseforge_query).hint_text("Search CurseForge modpacks"));
                            if ui.button("Search").clicked() {
                                match library::search_curseforge_modpacks(&self.curseforge_query) {
                                    Ok(results) => {
                                        self.curseforge_results = results;
                                        self.library_status = format!("Found {} CurseForge modpacks.", self.curseforge_results.len());
                                    }
                                    Err(error) => self.library_status = error,
                                }
                            }
                        });
                        for pack in self.curseforge_results.clone() {
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(RichText::new(&pack.title).strong().color(WHITE));
                                    ui.label(RichText::new(format!("{} downloads · {}", pack.downloads, pack.summary)).size(11.0).color(MUTED));
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui.button("Install").clicked() {
                                        self.library_status = match library::install_curseforge_project(pack.id) {
                                            Ok(message) => message,
                                            Err(error) => error,
                                        };
                                        self.instances = library::list_instances().unwrap_or_default();
                                    }
                                });
                            });
                        }
                    });
                    if !self.library_status.is_empty() {
                        ui.add_space(8.0);
                        ui.label(RichText::new(&self.library_status).color(BLUE));
                    }
                    ui.add_space(12.0);
                    Self::card(ui, |ui| {
                        ui.label(RichText::new(format!("INSTALLED MODPACKS  ·  {}", self.instances.len())).strong().color(WHITE));
                        ui.separator();
                        if self.instances.is_empty() {
                            ui.label(RichText::new("No modpacks installed. Import a Modrinth .mrpack to get started.").color(MUTED));
                        }
                        for name in self.instances.clone() {
                            ui.horizontal(|ui| {
                                let active = self.selected_instance.as_deref() == Some(name.as_str());
                                if ui.selectable_label(active, RichText::new(&name).color(WHITE)).clicked() {
                                    self.selected_instance = Some(name.clone());
                                    self.library_status = format!("{name} selected. Use Launch Minecraft on Home to start this instance.");
                                }
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui.small_button("Delete").clicked() {
                                        if self.selected_instance.as_deref() == Some(name.as_str()) { self.selected_instance = None; }
                                        self.library_status = match library::delete_instance(&name) {
                                            Ok(()) => format!("Deleted {name}."),
                                            Err(error) => error,
                                        };
                                        self.instances = library::list_instances().unwrap_or_default();
                                    }
                                });
                            });
                        }
                    });
                    ui.add_space(10.0);
                    ui.label(RichText::new(if let Some(name) = self.selected_instance.as_deref() {
                        format!("Active instance: {name}. Launch it from Home.")
                    } else {
                        "Default profile is active. Select an installed modpack to launch it.".to_owned()
                    }).size(11.0).color(MUTED));
                    ui.label(RichText::new("Support: Modrinth discovery / .mrpack import · CurseForge exported ZIP import · Fabric · Minecraft 26.2.").size(11.0).color(MUTED));
                }
                Page::Settings => {
                    ui.heading(RichText::new("Settings").size(28.0).strong().color(WHITE));
                    ui.label(RichText::new("Launcher configuration.").color(MUTED));
                    ui.add_space(20.0);
                    Self::card(ui, |ui| {
                        ui.label(RichText::new("Minecraft version").strong().color(WHITE));
                        ui.label(RichText::new("26.2 (fixed target)").color(BLUE));
                        ui.separator();
                        ui.label(RichText::new("Game directory").strong().color(WHITE));
                        ui.label(RichText::new("%APPDATA%/.minecraft").color(MUTED));
                        ui.separator();
                        ui.label(RichText::new("Authentication").strong().color(WHITE));
                        ui.label(RichText::new("Microsoft OAuth • Xbox Live • XSTS • Minecraft Services").color(MUTED));
                        ui.label(RichText::new("Session stored in Windows Credential Manager.").color(MUTED));
                    });
                }
                Page::About => {
                    ui.heading(RichText::new("VLOX CLIENT").size(28.0).strong().color(WHITE));
                    ui.label(RichText::new("A custom Minecraft Java 26.2 launcher.").color(MUTED));
                    ui.add_space(20.0);
                    Self::card(ui, |ui| {
                        ui.label(RichText::new("Built to feel personal.").size(19.0).strong().color(WHITE));
                        ui.label(RichText::new("VLOX handles Microsoft sign-in, Minecraft file installation, Fabric setup and direct game launching.").color(MUTED));
                        ui.label(RichText::new("Early access • 0.3.0").color(BLUE));
                    });
                }
            });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("VLOX CLIENT")
            .with_inner_size([1180.0, 800.0])
            .with_min_inner_size([900.0, 680.0])
            .with_decorations(false),
        ..Default::default()
    };
    eframe::run_native("VLOX CLIENT", options, Box::new(|_| Ok(Box::new(VloxApp::default()))))
}
