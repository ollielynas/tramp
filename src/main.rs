use egui::{ Id, plot::{PlotPoint, Plot, HLine, LineStyle, Points, PlotPoints, Line, Legend}, Color32 };
use macroquad::{ prelude::* };
use std::{ fs, io::{ self, stdout, Write }, ops::RangeInclusive, process::Stdio, time::Duration };
mod common_skills;
use common_skills::SKILLS;
use phf::phf_map;
extern crate savefile;
use savefile::prelude::*;
use std::time::{ Instant, UNIX_EPOCH };
#[macro_use]
extern crate savefile_derive;
use nfd2;
use std::env;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use chrono;

use ffmpeg_sidecar::{ self};

mod skill;
use skill::*;

mod video;
use video::*;


#[derive(PartialEq, Clone, Copy, Savefile, Debug)]
enum Tab {
    Edit,
    Info,
    DragAndDrop,
    Metadata,
}
#[derive(PartialEq, Clone, Savefile, Debug)]
struct Routine {
    skills: [Skill; 10],
    name: String,
    current_tab: Tab,
    id: String,
    #[savefile_default_fn = "false_func"]
    #[savefile_ignore]
    open:bool,
}

impl Routine {
    fn display(&mut self, egui_ctx: &egui::Context) {
        egui::Window
            ::new(format!("Routine: {}", self.name))
            .id(Id::new(&self.id))
            .scroll2([true, true])
            .open(&mut self.open)
            .show(egui_ctx, |ui| {
                let mut from = BodyPart::Feet;
                ui.horizontal(|ui| {
                    ui.label("Name: ");
                    ui.text_edit_singleline(&mut self.name);
                });
                ui.horizontal(|ui| {
                    if self.current_tab == Tab::Edit {
                        ui.label("Edit");
                    } else {
                        ui.small_button("Edit")
                            .clicked()
                            .then(|| {
                                self.current_tab = Tab::Edit;
                            });
                    }
                    ui.separator();
                    if self.current_tab == Tab::Info {
                        ui.label("Info");
                    } else {
                        ui.small_button("Info")
                            .clicked()
                            .then(|| {
                                self.current_tab = Tab::Info;
                            });
                    }
                    ui.separator();
                    if self.current_tab == Tab::DragAndDrop {
                        ui.label("Drag and Drop");
                    } else {
                        ui.small_button("Drag and Drop")
                            .clicked()
                            .then(|| {
                                self.current_tab = Tab::DragAndDrop;
                            });
                    }
                    ui.separator();
                    if self.current_tab == Tab::Metadata {
                        ui.label("Metadata");
                    } else {
                        ui.small_button("Metadata")
                            .clicked()
                            .then(|| {
                                self.current_tab = Tab::Metadata;
                            });
                    }
                });
                ui.separator();
                match self.current_tab {
                    Tab::Edit => {
                        for (i, skill) in self.skills.iter_mut().enumerate() {
                            ui.push_id(i, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{}: ", i + 1));
                                    from = skill.display(
                                        egui_ctx,
                                        ui,
                                        from,
                                        format!("{}{}", self.id, i)
                                    );
                                });
                            });
                        }
                    }
                    Tab::Info => {
                        ui.label(
                            format!(
                                "Total Difficulty: {:.2}",
                                self.skills
                                    .iter()
                                    .map(|s| s.diff())
                                    .sum::<f32>()
                            )
                        );
                        ui.separator();
                        let largest_rotation = self.skills
                            .iter()
                            .map(|s| s.flip)
                            .map(|i| (i * 4.0) as i32)
                            .max()
                            .unwrap_or(0);
                        ui.label(
                            format!(
                                "Largest Rotation {} ({} degrees)",
                                (largest_rotation as f32) / 4.0,
                                ((largest_rotation as f32) / 4.0) * 360.0
                            )
                        );
                        let largest_twist = self.skills
                            .iter()
                            .map(|s| s.twist.iter().sum::<f32>())
                            .map(|i| (i * 2.0) as i32)
                            .max()
                            .unwrap_or(0);
                        ui.label(
                            format!(
                                "Largest Twist {} ({} degrees)",
                                (largest_twist as f32) / 2.0,
                                ((largest_twist as f32) / 2.0) * 360.0
                            )
                        );
                    }
                    Tab::DragAndDrop => {
                        ui.label("Drag and Drop");
                    }
                    Tab::Metadata => {
                        ui.label(format!("Id: {}", self.id));
                        let root = match env::current_dir() {
                            Ok(path) => path.display().to_string(),
                            Err(a) => format!("{a}"),
                        };

                        ui.label(format!("Root: {root}"));
                        ui.label(format!("Path: {root}/routines/{}.bin", self.id));
                    }
                }
                ui.add_sized(ui.available_size(), egui::Label::new(""))
            });
    }
    fn blank() -> Routine {
        Routine {
            skills: [
                Skill::from_notation("0 o".to_owned(), BodyPart::Feet).unwrap(),
                Skill::from_notation("0 o".to_owned(), BodyPart::Feet).unwrap(),
                Skill::from_notation("0 o".to_owned(), BodyPart::Feet).unwrap(),
                Skill::from_notation("0 o".to_owned(), BodyPart::Feet).unwrap(),
                Skill::from_notation("0 o".to_owned(), BodyPart::Feet).unwrap(),
                Skill::from_notation("0 o".to_owned(), BodyPart::Feet).unwrap(),
                Skill::from_notation("0 o".to_owned(), BodyPart::Feet).unwrap(),
                Skill::from_notation("0 o".to_owned(), BodyPart::Feet).unwrap(),
                Skill::from_notation("0 o".to_owned(), BodyPart::Feet).unwrap(),
                Skill::from_notation("0 o".to_owned(), BodyPart::Feet).unwrap(),
            ],
            name: "New Routine".to_owned(),
            current_tab: Tab::Edit,
            id: UNIX_EPOCH.elapsed().unwrap().as_secs_f32().to_string().replace(".", ""),
            open: true,
        }
    }
}

#[derive(Debug, Clone, EnumIter, Savefile, PartialEq, Eq, Copy)]
enum WindowTheme {
    Light,
    Dark,
    Latte,
    Frappe,
    Macchiato,
    Mocha,
}
impl WindowTheme {
    fn set_theme(&self, egui_ctx: &egui::Context) {
        match self {
            WindowTheme::Light => {
                egui_ctx.set_visuals(egui::Visuals::light());
            }
            WindowTheme::Dark => {
                egui_ctx.set_visuals(egui::Visuals::dark());
            }
            WindowTheme::Mocha => {
                catppuccin_egui::set_theme(&egui_ctx, catppuccin_egui::MOCHA);
            }
            WindowTheme::Latte => {
                catppuccin_egui::set_theme(&egui_ctx, catppuccin_egui::LATTE);
            }
            WindowTheme::Frappe => {
                catppuccin_egui::set_theme(&egui_ctx, catppuccin_egui::FRAPPE);
            }
            WindowTheme::Macchiato => {
                catppuccin_egui::set_theme(&egui_ctx, catppuccin_egui::MACCHIATO);
            }
        }
    }

    fn bg(&self) -> macroquad::color::Color {
        match self {
            WindowTheme::Light => macroquad::color::Color::new(0.9, 0.9, 0.9, 1.0),
            WindowTheme::Dark => macroquad::color::Color::new(0.1, 0.1, 0.1, 1.0),
            WindowTheme::Mocha => {
                let r = catppuccin_egui::MOCHA.crust[0];
                let g = catppuccin_egui::MOCHA.crust[1];
                let b = catppuccin_egui::MOCHA.crust[2];
                macroquad::color::Color::new(
                    (r as f32) / 256.0,
                    (g as f32) / 256.0,
                    (b as f32) / 256.0,
                    1.0
                )
            }
            WindowTheme::Latte => {
                let r = catppuccin_egui::LATTE.crust[0];
                let g = catppuccin_egui::LATTE.crust[1];
                let b = catppuccin_egui::LATTE.crust[2];
                macroquad::color::Color::new(
                    (r as f32) / 256.0,
                    (g as f32) / 256.0,
                    (b as f32) / 256.0,
                    1.0
                )
            }
            WindowTheme::Frappe => {
                let r = catppuccin_egui::FRAPPE.crust[0];
                let g = catppuccin_egui::FRAPPE.crust[1];
                let b = catppuccin_egui::FRAPPE.crust[2];
                macroquad::color::Color::new(
                    (r as f32) / 256.0,
                    (g as f32) / 256.0,
                    (b as f32) / 256.0,
                    1.0
                )
            }
            WindowTheme::Macchiato => {
                let r = catppuccin_egui::MACCHIATO.crust[0];
                let g = catppuccin_egui::MACCHIATO.crust[1];
                let b = catppuccin_egui::MACCHIATO.crust[2];
                macroquad::color::Color::new(
                    (r as f32) / 256.0,
                    (g as f32) / 256.0,
                    (b as f32) / 256.0,
                    1.0
                )
            }
        }
    }
}

struct Data {
    routines: Vec<Routine>,
    theme: WindowTheme,
    judged: Vec<Judged>,
    zoom: f32,
}

impl Data {
    fn render(&mut self, egui_ctx: &egui::Context) {
        for r in self.routines.iter_mut() {
            r.display(&egui_ctx);
        }
        for r in self.judged.iter_mut() {
            if r.open {
            if r.routine_id == "" {
                egui::Window::new("Select Routine").show(egui_ctx, |ui| {
                    ui.label("Select a routine to judge");
                    ui.separator();
                    for i in self.routines.iter() {
                        if ui.button(&i.name).clicked() {
                            r.routine_id = i.id.clone();
                        }
                    }
                });
            }else {
            r.display(&egui_ctx);
            }
        }

        }
    }

    fn save(&self) {
        match fs::create_dir_all("./Data/routines") {
            Ok(_) => {}
            Err(e) => {
                error!("Error creating directory: {}", e);
            }
        }
        match fs::create_dir_all("./Data/judge") {
            Ok(_) => {}
            Err(e) => {
                error!("Error creating directory: {}", e);
            }
        }
        for i in self.judged.iter() {
            match savefile::save_file(format!("Data/judge/{}.bin", i.id), 1, i) {
                Ok(_) => {}
                Err(e) => {
                    error!("Error saving file: {}", e);
                }
            }
        }
        for i in &self.routines {
            match savefile::save_file(format!("Data/routines/{}.bin", i.id), 1, i) {
                Ok(_) => {}
                Err(e) => {
                    error!("Error saving file: {}", e);
                }
            }
        }
        match savefile::save_file("Data/theme.bin", 1, &self.theme) {
            Ok(_) => {}
            Err(e) => { error!("Error saving file: {}", e) }
        }
        match savefile::save_file("Data/zoom.bin", 1, &self.zoom) {
            Ok(_) => {}
            Err(e) => { error!("Error saving file: {}", e) }
        }
    }

    fn load_files(&mut self) {
        match savefile::load_file("Data/theme.bin", 1) {
            Ok(theme) => {
                self.theme = theme;
            }
            Err(e) => {
                error!("Error loading file: {}", e);
            }
        }
        match savefile::load_file("Data/zoom.bin", 1) {
            Ok(zoom) => {
                self.zoom = zoom;
            }
            Err(e) => {
                error!("Error loading file: {}", e);
            }
        }
        self.routines.clear();
        for file in match fs::read_dir("./Data/routines") {Ok(file) => file,Err(e) => {println!("{e}");return}} {
            let file = match file {
                Ok(file) => file,
                Err(e) => {
                    error!("Error reading file: {}", e);
                    continue;
                }
            };
            let path = file.path();
            let path = match path.to_str() {
                Some(path) => path,
                None => {
                    error!("Error reading path");
                    continue;
                }
            };
            match savefile::load_file(path, 1) {
                Ok(routine) => self.routines.push(routine),
                Err(e) => {
                    error!("Error loading file: {}", e);
                    continue;
                }
            };
        }
        self.judged.clear();
        for file in match fs::read_dir("./Data/judge") {Ok(file) => file,Err(e) => {println!("{e}");return}} {
            let file = match file {
                Ok(file) => file,
                Err(e) => {
                    error!("Error reading file: {}", e);
                    return;
                }
            };
            let path = file.path();
            let path = match path.to_str() {
                Some(path) => path,
                None => {
                    error!("Error reading path");
                    return;
                }
            };
            match savefile::load_file(path, 1) {
                Ok(routine) => self.judged.push(routine),
                Err(e) => {
                    error!("Error loading file: {}", e);
                    return;
                }
            };
        }
    }
}

#[derive(Debug, Clone, Savefile, Eq, PartialEq)]
enum Panel {
    Totals,
    Routine,
    Diff,
    HD,
    TOF,
    Execution,
    Deductions,
    Graph,

}

impl Default for Panel {
    fn default() -> Self {
        Panel::Routine
    }
}

fn none_routine() -> Option<Routine> {
    None
}

fn false_func() -> bool{false}

#[derive(Debug, Clone, Savefile)]
struct Judged {
    #[savefile_default_fn = "false_func"]
    #[savefile_ignore]
    open:bool,
    #[savefile_default_fn = "none_routine"]
    #[savefile_ignore]
    routine: Option<Routine>,
    #[savefile_ignore]
    panel: Panel,
    routine_id: String,
    execution_1: [f32;10],
    execution_5: [[f32;10];5],
    five_judges: bool,
    date_of_creation: String,
    hd: [f32;10],
    id: String,
    tof_total: f32,
    tof: [f32;10],

}

impl Judged {
    fn new() -> Judged {
        Judged {
            routine: None,
            panel: Panel::Routine,
            routine_id: String::new(),
            execution_1: [0.0;10],
            execution_5: [[0.0;10];5],
            five_judges: false,
            hd: [0.0;10],
            id: UNIX_EPOCH.elapsed().unwrap().as_secs_f32().to_string().replace(".", ""),
            open: true,
            date_of_creation: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
            tof_total: 0.0,
            tof: [0.0;10],
        }
    }

    fn display(&mut self, egui_ctx:&egui::Context) {
        egui::Window::new(format!("Judged Routine: {}", self.id))
            .open(&mut self.open)
            .id(Id::new(&self.id)).show(egui_ctx, |ui| {
            egui::ScrollArea::horizontal().show(ui, |ui| {

            if self.routine.is_none() {
                match savefile::load_file(format!("Data/routines/{}.bin", self.routine_id), 1) {
                    Ok(routine) => {
                        self.routine = Some(routine);
                    }
                    Err(e) => {
                        error!("Error loading file: {}", e);
                    }
                }
            }
            egui::ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal(|ui| {
                    if self.panel == Panel::Routine {
                        ui.label("Routine");
                    } else {
                        ui.small_button("Routine")
                            .clicked()
                            .then(|| {
                                self.panel = Panel::Routine;
                            });
                    }
                    if self.panel == Panel::Graph {
                        ui.label("Graph");
                    } else {
                        ui.small_button("Graph")
                            .clicked()
                            .then(|| {
                                self.panel = Panel::Graph;
                            });
                    }
                    ui.separator();
                    if self.panel == Panel::Diff {
                        ui.label("Difficulty");
                    } else {
                        ui.small_button("Difficulty")
                            .clicked()
                            .then(|| {
                                self.panel = Panel::Diff;
                            });
                    }
                    ui.separator();
                    if self.panel == Panel::HD {
                        ui.label("HD");
                    } else {
                        ui.small_button("HD")
                            .clicked()
                            .then(|| {
                                self.panel = Panel::HD;
                            });
                    }
                    ui.separator();
                    if self.panel == Panel::TOF {
                        ui.label("TOF");
                    } else {
                        ui.small_button("TOF")
                            .clicked()
                            .then(|| {
                                self.panel = Panel::TOF;
                            });
                    }
                    ui.separator();
                    if self.panel == Panel::Execution {
                        ui.label("Execution");
                    } else {
                        ui.small_button("Execution")
                            .clicked()
                            .then(|| {
                                self.panel = Panel::Execution;
                            });
                    }
                    ui.separator();
                    if self.panel == Panel::Deductions {
                        ui.label("Deductions");
                    } else {
                        ui.small_button("Deductions")
                            .clicked()
                            .then(|| {
                                self.panel = Panel::Deductions;
                            });
                    }
                    ui.separator();
                    if self.panel == Panel::Totals {
                        ui.label("Totals");
                    } else {
                        ui.small_button("Totals")
                            .clicked()
                            .then(|| {
                                self.panel = Panel::Totals;
                            });
                    }
                    
                });
            });
            ui.separator();

            match &self.panel {
                &Panel::Routine => {
                    ui.small_button("reload").clicked().then(||{
                        match savefile::load_file(format!("Data/routines/{}.bin", self.routine_id), 1) {
                    Ok(routine) => {
                        self.routine = Some(routine);
                    }
                    Err(e) => {
                        error!("Error loading file: {}", e);
                    }
                }
                        
                    });
                        for i in 0..10 {
                        ui.label(format!("{}.) {}",i+1,self.routine.as_ref().unwrap().skills[i].name()));
                    }
                    
                }
                &Panel::Diff => {
                    ui.label(format!("Total Difficulty: +{:.2}" ,(0..10).map(|i|(self.routine.as_ref().unwrap().skills[i].diff()*100.0)as i32).sum::<i32>()as f32 /100.0));
                    ui.separator();
                    for i in 0..10 {
                    ui.label(format!("{}.) {}",i+1,self.routine.as_ref().unwrap().skills[i].diff()));
                    }

                }
                &Panel::HD => {
                    let mut total = 0.0;
                    for i in 0..10 {
                        total += self.hd[i];
                    }
                    ui.heading(format!("Total HD: -{:.2}" ,total));
                    for i in 0..10 {
                        ui.label(format!("{}.) {}",i,self.routine.as_ref().unwrap().skills[i].name()));
                        ui.horizontal(|ui| {
                            ui.selectable_label(self.hd[i] == 0.0, "0.0").clicked().then(|| {self.hd[i] = 0.0});
                            ui.selectable_label(self.hd[i] == 0.1, "0.1").clicked().then(|| {self.hd[i] = 0.1});
                            ui.selectable_label(self.hd[i] == 0.2, "0.2").clicked().then(|| {self.hd[i] = 0.2});
                            ui.selectable_label(self.hd[i] == 0.3, "0.3").clicked().then(|| {self.hd[i] = 0.3});
                        });
                    }
                }
                &Panel::Execution => {
                    ui.label(format!("Execution: -{:.2}", match self.five_judges {
                        false => {self.execution_1.iter().sum::<f32>()},
                        true => {
                            let mut totals = self.execution_5.iter().map(|x| x.iter().sum::<f32>()).collect::<Vec<f32>>();
                            totals.sort_by(|a,b| a.partial_cmp(b).unwrap());
                            totals[1..4].iter().sum::<f32>()
                        }
                    }));
                    if self.five_judges {
                        for i in 0..10 {
                        ui.selectable_label(self.execution_1[i] == 0.0, "0.0").clicked().then(|| {self.hd[i] = 0.0});
                            ui.separator();
                            ui.label("Unfinished");
                        }
                    }else{
                        let mut total = 0.0;
                        for i in 0..10 {
                        total += self.hd[i];
                    }
                    total = total*10.0;
                    ui.label(format!("Total HD: -{:.2}" ,total));
                    for i in 0..10 {
                        ui.label(format!("{}.) {}",i,self.routine.as_ref().unwrap().skills[i].name()));
                        ui.horizontal(|ui| {
                                ui.selectable_label(self.execution_1[i] == 0.0, "0.0").clicked().then(|| {self.execution_1[i] = 0.0});
                                ui.selectable_label(self.execution_1[i] == 0.1, "0.1").clicked().then(|| {self.execution_1[i] = 0.1});
                                ui.selectable_label(self.execution_1[i] == 0.2, "0.2").clicked().then(|| {self.execution_1[i] = 0.2});
                                ui.selectable_label(self.execution_1[i] == 0.3, "0.3").clicked().then(|| {self.execution_1[i] = 0.3});
                                ui.selectable_label(self.execution_1[i] == 0.4, "0.4").clicked().then(|| {self.execution_1[i] = 0.4});
                                ui.selectable_label(self.execution_1[i] == 0.5, "0.5").clicked().then(|| {self.execution_1[i] = 0.5});
                        });
                    }
                }
                }
                &Panel::Deductions => {
                    ui.label("todo");
                }
                &Panel::Totals => {
                    ui.label("todo");
                }
                &Panel::TOF => {
                            ui.horizontal(|ui| {
                                ui.label("Total TOF ");
                            ui.add(egui::DragValue::new(&mut self.tof_total).speed(0.1).clamp_range(0.0..=50.0).fixed_decimals(2)
                                    .suffix("sec"));
                            });
                            ui.separator();
                            for i in 0..10 {
                                ui.horizontal(|ui| {
                                ui.label(format!("{}.) ",i+1));
                                if ui.add(egui::DragValue::new(&mut self.tof[i]).speed(0.01).clamp_range(0.0..=5.0)
                                    .suffix("sec")).changed() {
                                        self.tof_total = self.tof.iter().sum::<f32>();
                                    };
                                });
                            }
                }
                &Panel::Graph => {

                        fn point_label(p: f64, _range: &RangeInclusive<f64>) -> String {
                            format!("{p}")
                        }

                        fn label_formatter(name: &str, p: &PlotPoint) -> String {
                            format!("{} {:.2}", name, p.x)
                        }
                        

                        Plot::new("my_plot")
                            .label_formatter(label_formatter)
                            .x_axis_formatter(point_label)
                            .height(ui.available_height()*0.75)
                            .legend(Legend::default())
                            .show(ui, |plot_ui| {
                                plot_ui.hline(HLine::new(self.tof_total/10.0).name("average time").color(Color32::RED).style(LineStyle::Dashed { length: 5.0 }));
                                let points: PlotPoints  = (0..10).map(|i| [i as f64, self.tof[i] as f64]).collect();
                                plot_ui.line(Line::new(points).color(Color32::RED).name("time of flight (sec)"));

                                if self.five_judges {
                                    for k in 0..5 {
                                        plot_ui.hline(HLine::new(self.execution_5[k].iter().sum::<f32>()/10.0).name("average time").color(Color32::BLUE).style(LineStyle::Dashed { length: 5.0 }));
                                let points: PlotPoints  = (0..10).map(|i| [i as f64, self.execution_5[k][i] as f64]).collect();
                                plot_ui.line(Line::new(points).color(Color32::BLUE).name("Execution"));
                                    }
                                }else{
                                plot_ui.hline(HLine::new(self.execution_1.iter().sum::<f32>()/10.0).name("average time").color(Color32::BLUE).style(LineStyle::Dashed { length: 5.0 }));
                                let points: PlotPoints  = (0..10).map(|i| [i as f64, self.execution_1[i] as f64]).collect();
                                plot_ui.line(Line::new(points).color(Color32::BLUE).name("Execution"));
                                }
                            });
                        
                }
            }
                
        });
        ui.add_sized(ui.available_size(), egui::Label::new(""))
    });
    }
    

}

#[macroquad::main("Trampoline thing")]
async fn main() {
    let mut  og_ppp = 0.0;
    let mut now = Instant::now();
    // get text input from user
    let mut data = Data {
        zoom: 1.0,
        routines: vec![],
        theme: WindowTheme::Light,
        judged: vec![],
    };
    
    // let mut
    
    let mut fonts = egui::FontDefinitions::default();
    ffmpeg_sidecar::download::auto_download().unwrap();
    
    data.load_files();
    egui_macroquad::ui(|egui_ctx| {
        data.theme.set_theme(egui_ctx);
        og_ppp = egui_ctx.pixels_per_point();
        egui_phosphor::add_to_fonts(&mut fonts);
        egui_ctx.set_pixels_per_point(og_ppp * data.zoom);

    egui_ctx.set_fonts(fonts);
    });

    let mut videos: Vec<Video> = vec![];

    loop {
        if now.elapsed().as_millis() > 1000 {
            data.save();
            now = Instant::now();
        }

        clear_background(data.theme.bg());
        // Process keys, mouse etc.
        egui_macroquad::ui(|egui_ctx| {
            data.theme.set_theme(egui_ctx);

            egui::SidePanel::left("Left").show(egui_ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Routines");
                ui.separator();
                ui.button(egui_phosphor::icons::PLUS.to_owned()+" New Routine")
                    .clicked()
                    .then(|| {
                        data.routines.push(Routine::blank());
                    });
                ui.menu_button(egui_phosphor::icons::PENCIL.to_owned() + " Edit Routine", |ui| {
                    for r in data.routines.iter_mut() {
                        let toggle = !r.open;
                        ui.selectable_value(&mut r.open, toggle, &r.name);
                    }
                });
                
                ui.button(format!("{} Judge Routine", egui_phosphor::BOOK_BOOKMARK))
                    .clicked()
                    .then(|| {
                        data.judged.push(Judged::new());
                    });
                    
                ui.collapsing( "Past Routines", |ui| {
                    let mut delete:Vec<usize> = vec![];
                    for (i, r) in data.judged.iter_mut().enumerate() {
                        let toggle = !r.open;
                        ui.horizontal(|ui| {
                            ui.selectable_value(&mut r.open, toggle, match &r.routine {Some(a) => &a.name, None => "None"});
                            ui.small_button("Delete").on_hover_text("Waring! Permanent").clicked().then(|| {
                                delete.push(i);
                                match fs::remove_file(format!("Data/judged/{}.bin", r.id)) {
                                    Ok(_) => {},
                                    Err(e) => {
                                        error!("Error deleting file: {}", e);
                                    }
                                };
                            })
                        });
                        ui.label(&r.date_of_creation);
                        ui.separator();
                    };
                }).header_response.clicked().then(|| {
                    for i in data.judged.iter_mut() {
                        if i.routine.is_none() {
                match savefile::load_file(format!("Data/routines/{}.bin", i.routine_id), 1) {
                    Ok(routine) => {
                        i.routine = Some(routine);
                    }
                    Err(e) => {
                        error!("Error loading file: {}", e);
                    }
                }
            }
                    }
                });
                ui.heading("Video");
                ui.separator();
                ui.button(format!("{} load video", egui_phosphor::FILE_VIDEO))
                    .clicked()
                    .then(|| {
                        videos.push(Video::new());
                    });
                ui.menu_button("reopen video", |ui| {
                    for r in videos.iter_mut() {
                        let toggle = !r.open;
                        ui.selectable_value(&mut r.open, toggle, &r.path);
                    }
                });
                ui.button(format!("{} Update/download ffmpeg", egui_phosphor::CLOUD_ARROW_DOWN))
                    .clicked()
                    .then(|| {
                        ffmpeg_sidecar::download::auto_download().unwrap();
                    });
                ui.heading("Files");
                ui.separator();
                ui.button(format!("{} Save", egui_phosphor::FLOPPY_DISK))
                    .clicked()
                    .then(|| {
                        data.save();
                    });
                ui.heading("Settings");
                ui.separator();
                ui.menu_button(format!("{} UI Style", egui_phosphor::PALETTE), |ui| {
                    for s in WindowTheme::iter() {
                        ui.selectable_value(&mut data.theme, s, &format!("{:?}", s));
                    }
                })
                    .response.clicked()
                    .then(|| {
                        data.theme.set_theme(egui_ctx);
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Zoom");
                ui.small_button(egui_phosphor::MAGNIFYING_GLASS_MINUS).clicked().then(|| {
                    data.zoom -= 0.1;
                    egui_ctx.set_pixels_per_point(og_ppp * data.zoom);
                });
                ui.label(&format!("{:.1}", data.zoom));
                ui.small_button(egui_phosphor::MAGNIFYING_GLASS_PLUS).clicked().then(|| {
                    data.zoom += 0.1;
                    egui_ctx.set_pixels_per_point(og_ppp * data.zoom)
                });
                
            });

            for i in videos.iter_mut() {
                i.full_size = false;
                if i.open {
                    i.display(egui_ctx);
                    if i.kill {
                        for t in i.textures.iter_mut() {
                            t.0.delete();
                        }
                        i.textures.clear();
                    }
                }
            }
            data.render(&egui_ctx);
        });
        });
        videos.retain(|x| !x.kill);

        egui_macroquad::draw();
        for v in &videos {
            if v.show_video && v.open {
                if v.textures.len() == 0 {
                    break
                }
                let index = clamp(v.current_frame, 0, v.textures.len() - 1);
                let frame = &v.textures[index];
                draw_texture_ex(frame.0, v.rect[0] * data.zoom, v.rect[1] * data.zoom, WHITE, DrawTextureParams {
                    dest_size: Some(vec2(v.rect[2]*data.zoom, v.rect[3]*data.zoom)),
                    ..Default::default()
                });
            }
        }

        next_frame().await;
    }
}