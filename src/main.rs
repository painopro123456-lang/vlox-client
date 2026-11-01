use eframe::egui::{self, Align, Color32, FontId, Layout, RichText, Rounding, Stroke, Vec2};
use std::{fs, path::PathBuf, process::Command};

const BG: Color32 = Color32::from_rgb(10, 13, 20);
const PANEL: Color32 = Color32::from_rgb(18, 24, 36);
const BLUE: Color32 = Color32::from_rgb(76, 139, 255);
const MUTED: Color32 = Color32::from_rgb(145, 158, 180);
const WHITE: Color32 = Color32::from_rgb(238, 243, 252);

#[derive(Clone, Copy, PartialEq)] enum Page { Home, Mods, Settings, About }
struct VloxApp { page: Page, status: String, version: String }
impl Default for VloxApp {
 fn default()->Self { Self { page:Page::Home, status:"Ready when you are.".into(), version:"1.21.1".into() } }
}
impl VloxApp {
 fn card<R>(ui:&mut egui::Ui,f:impl FnOnce(&mut egui::Ui)->R)->R {
  egui::Frame::none().fill(PANEL).rounding(Rounding::same(15.0)).stroke(Stroke::new(1.0,Color32::from_rgb(37,47,65))).inner_margin(egui::Margin::same(18.0)).show(ui,f).inner
 }
 fn nav(&mut self,ui:&mut egui::Ui,label:&str,page:Page) {
  let active=self.page==page;
  if ui.add_sized([190.0,42.0],egui::Button::new(RichText::new(label).size(14.0).color(if active {WHITE}else{MUTED})).fill(if active {Color32::from_rgb(30,49,79)}else{Color32::TRANSPARENT}).rounding(9.0).stroke(Stroke::NONE)).clicked(){self.page=page;}
 }
 fn install(&mut self) {
  let source=PathBuf::from("client-mod/build/libs/vlox-client-mod-0.1.0.jar");
  if !source.exists(){self.status="Build the Fabric mod first (GitHub Actions → VLOX-CLIENT-Fabric-1.21.1).".into();return;}
  let Some(appdata)=std::env::var_os("APPDATA") else {self.status="Windows AppData folder was not found.".into();return;};
  let dest=PathBuf::from(appdata).join(".minecraft").join("mods");
  if let Err(e)=fs::create_dir_all(&dest){self.status=format!("Could not create mods folder: {e}");return;}
  self.status=match fs::copy(source,dest.join("vlox-client-mod.jar")){Ok(_)=>"VLOX mod installed. Launch the matching Fabric profile.".into(),Err(e)=>format!("Install failed: {e}")};
 }
 fn open_game(&mut self) {
  #[cfg(target_os="windows")] let r=Command::new("cmd").args(["/C","start","","minecraft:"]).spawn();
  #[cfg(target_os="macos")] let r=Command::new("open").arg("minecraft:").spawn();
  #[cfg(all(not(target_os="windows"),not(target_os="macos")))] let r=Command::new("xdg-open").arg("minecraft:").spawn();
  self.status=match r{Ok(_)=>"Opening the Minecraft Launcher…".into(),Err(e)=>format!("Could not open launcher: {e}")};
 }
}
impl eframe::App for VloxApp {
 fn update(&mut self,ctx:&egui::Context,_:&mut eframe::Frame){
  ctx.set_visuals(egui::Visuals::dark());
  let mut s=(*ctx.style()).clone();s.text_styles.insert(egui::TextStyle::Body,FontId::proportional(14.0));s.visuals.window_fill=BG;s.visuals.panel_fill=BG;s.spacing.item_spacing=Vec2::new(10.0,10.0);ctx.set_style(s);
  egui::SidePanel::left("side").exact_width(224.0).frame(egui::Frame::none().fill(Color32::from_rgb(12,16,24)).inner_margin(egui::Margin::same(16.0))).show(ctx,|ui|{
   ui.add_space(10.0);ui.horizontal(|ui|{ui.label(RichText::new("V").size(30.0).strong().color(BLUE));ui.vertical(|ui|{ui.label(RichText::new("VLOX").size(19.0).strong().color(WHITE));ui.label(RichText::new("CLIENT").size(10.0).strong().color(MUTED));});});
   ui.add_space(34.0);ui.label(RichText::new("WORKSPACE").size(10.0).strong().color(MUTED));ui.add_space(8.0);
   self.nav(ui,"⌂   Home",Page::Home);self.nav(ui,"▦   Mod Library",Page::Mods);self.nav(ui,"⚙   Settings",Page::Settings);self.nav(ui,"ⓘ   About VLOX",Page::About);
   ui.with_layout(Layout::bottom_up(Align::Min),|ui|{ui.separator();ui.label(RichText::new("VLOX CLIENT  •  EARLY ACCESS").size(9.0).color(MUTED));ui.horizontal(|ui|{ui.colored_label(Color32::from_rgb(87,205,139),"●");ui.label(RichText::new("Desktop ready").size(11.0).color(MUTED));});});
  });
  egui::CentralPanel::default().frame(egui::Frame::none().fill(BG).inner_margin(egui::Margin::same(28.0))).show(ctx,|ui|match self.page{
   Page::Home=>{
    ui.label(RichText::new("YOUR GAME. YOUR WAY.").size(10.0).strong().color(BLUE));ui.add_space(7.0);ui.heading(RichText::new("Welcome to VLOX.").size(30.0).strong().color(WHITE));ui.label(RichText::new("A cleaner, more personal way to play Minecraft Java.").color(MUTED));ui.add_space(22.0);
    egui::Frame::none().fill(Color32::from_rgb(18,31,52)).rounding(Rounding::same(18.0)).stroke(Stroke::new(1.0,Color32::from_rgb(38,66,105))).inner_margin(egui::Margin::same(24.0)).show(ui,|ui|{
     ui.set_min_height(170.0);ui.horizontal(|ui|{ui.vertical(|ui|{ui.label(RichText::new("MADE FOR YOUR NEXT WORLD").size(10.0).strong().color(BLUE));ui.add_space(8.0);ui.label(RichText::new("Make Minecraft\nfeel like yours.").size(27.0).strong().color(WHITE));ui.add_space(14.0);if ui.add_sized([180.0,40.0],egui::Button::new(RichText::new("Open Minecraft").strong()).fill(BLUE).rounding(9.0)).clicked(){self.open_game();}});ui.with_layout(Layout::right_to_left(Align::Center),|ui|{ui.label(RichText::new("V").size(116.0).strong().color(Color32::from_rgb(48,83,137)));});});
    });ui.add_space(18.0);
    ui.columns(2,|c|{
     Self::card(&mut c[0],|ui|{ui.label(RichText::new("IN-GAME CLIENT").size(10.0).strong().color(MUTED));ui.label(RichText::new("VLOX Interface").size(19.0).strong().color(WHITE));ui.label(RichText::new("HUD, modules and visual settings inside Minecraft.").color(MUTED));ui.add_space(10.0);if ui.button("Install client mod").clicked(){self.install();}});
     Self::card(&mut c[1],|ui|{ui.label(RichText::new("GAME PROFILE").size(10.0).strong().color(MUTED));ui.label(RichText::new("Minecraft Java").size(19.0).strong().color(WHITE));ui.label(RichText::new("Fabric client profile").color(MUTED));ui.add_space(8.0);ui.horizontal(|ui|{ui.label("Target");egui::ComboBox::from_id_source("version").selected_text(&self.version).show_ui(ui,|ui|{for v in ["1.21.1","1.20.1"]{ui.selectable_value(&mut self.version,v.into(),v);}});});});
    });ui.add_space(16.0);Self::card(ui,|ui|{ui.label(RichText::new("ACTIVITY").size(10.0).strong().color(MUTED));ui.label(RichText::new(&self.status).color(WHITE));});
   },
   Page::Mods=>{ui.heading(RichText::new("Mod Library").size(28.0).strong().color(WHITE));ui.label(RichText::new("Manage your VLOX in-game client.").color(MUTED));ui.add_space(20.0);Self::card(ui,|ui|{ui.horizontal(|ui|{ui.label(RichText::new("VLOX In-game Client").size(18.0).strong().color(WHITE));ui.with_layout(Layout::right_to_left(Align::Center),|ui|{ui.colored_label(Color32::from_rgb(87,205,139),"● Fabric");});});ui.label(RichText::new("Client UI • HUD • module toggles").color(MUTED));ui.add_space(12.0);if ui.button("Install to .minecraft/mods").clicked(){self.install();}});},
   Page::Settings=>{ui.heading(RichText::new("Settings").size(28.0).strong().color(WHITE));ui.label(RichText::new("Desktop companion preferences.").color(MUTED));ui.add_space(20.0);Self::card(ui,|ui|{ui.label(RichText::new("Minecraft version").strong().color(WHITE));egui::ComboBox::from_id_source("settings-version").selected_text(&self.version).show_ui(ui,|ui|{for v in ["1.21.1","1.20.1"]{ui.selectable_value(&mut self.version,v.into(),v);}});ui.separator();ui.label(RichText::new("Game directory").strong().color(WHITE));ui.label(RichText::new("%APPDATA%/.minecraft").color(MUTED));});},
   Page::About=>{ui.heading(RichText::new("VLOX CLIENT").size(28.0).strong().color(WHITE));ui.label(RichText::new("A custom Minecraft Java client experience.").color(MUTED));ui.add_space(20.0);Self::card(ui,|ui|{ui.label(RichText::new("Built to feel personal.").size(19.0).strong().color(WHITE));ui.label(RichText::new("VLOX combines a desktop companion with an in-game Fabric client. It is an independent client-side add-on and does not bundle Minecraft assets.").color(MUTED));ui.label(RichText::new("Early access • 0.2.0").color(BLUE));});}
  });
 }
}
fn main()->eframe::Result<()>{
 let opts=eframe::NativeOptions{viewport:egui::ViewportBuilder::default().with_title("VLOX CLIENT").with_inner_size([1180.0,760.0]).with_min_inner_size([900.0,620.0]),..Default::default()};
 eframe::run_native("VLOX CLIENT",opts,Box::new(|_|Ok(Box::new(VloxApp::default()))))
}
