use egui::{ widgets, Id, Pos2 };
use macroquad::{ prelude::*, time };
use std::{ fs, io::{ self, stdout, Write }, ops::RangeInclusive, process::Stdio, time::Duration };
mod common_skills;
use common_skills::SKILLS;
use phf::phf_map;
extern crate savefile;
use savefile::prelude::*;
use std::time::{ Instant, UNIX_EPOCH };
#[macro_use]
extern crate savefile_derive;
use nfd2::Response;
use std::env;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

use ffmpeg_sidecar::{ self, command::FfmpegCommand, event::FfmpegEvent };

#[derive(PartialEq, Eq, Hash, Clone, Copy, Savefile, Debug)]
enum Shape {
    Straight,
    Pike,
    Tuck,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Savefile, Debug)]
enum BodyPart {
    Feet,
    Front,
    Back,
    Head,
    Seat,
}
#[derive(PartialEq, Eq, Hash, Clone, Copy, Savefile, Debug)]
enum FlipDirection {
    Forward,
    Backward,
}

impl BodyPart {
    fn name(&self) -> String {
        (
            match self {
                BodyPart::Feet => "feet",
                BodyPart::Front => "front",
                BodyPart::Back => "back",
                BodyPart::Head => "head",
                BodyPart::Seat => "seat",
            }
        ).to_owned()
    }
    fn add(&self, amount: f32, direction: FlipDirection, total_twist: f32) -> BodyPart {
        let direction = match (direction, (total_twist.fract() * 10.0) as i32) {
            (FlipDirection::Forward, 0) => FlipDirection::Forward,
            (FlipDirection::Forward, 5) => FlipDirection::Backward,
            (FlipDirection::Backward, 0) => FlipDirection::Backward,
            (FlipDirection::Backward, 5) => FlipDirection::Forward,
            _ => FlipDirection::Forward,
        };
        if amount == 0.0 {
            self.clone()
        } else if amount == 0.5 {
            match &self {
                BodyPart::Back => BodyPart::Front,
                BodyPart::Front => BodyPart::Back,
                BodyPart::Head => BodyPart::Feet,
                BodyPart::Feet => BodyPart::Head,
                BodyPart::Seat => BodyPart::Head,
            }
        } else if amount == 0.25 {
            match &self {
                BodyPart::Back | BodyPart::Front => BodyPart::Feet,
                BodyPart::Feet if direction == FlipDirection::Forward => BodyPart::Front,
                BodyPart::Feet if direction == FlipDirection::Backward => BodyPart::Back,
                BodyPart::Head if direction == FlipDirection::Forward => BodyPart::Back,
                BodyPart::Head if direction == FlipDirection::Backward => BodyPart::Front,
                BodyPart::Seat if direction == FlipDirection::Forward => BodyPart::Back,
                BodyPart::Seat if direction == FlipDirection::Backward => BodyPart::Front,
                _ => BodyPart::Feet,
            }
        } else {
            match &self {
                BodyPart::Back | BodyPart::Front => BodyPart::Feet,
                BodyPart::Feet if direction == FlipDirection::Forward => BodyPart::Back,
                BodyPart::Feet if direction == FlipDirection::Backward => BodyPart::Front,
                BodyPart::Seat if direction == FlipDirection::Forward => BodyPart::Back,
                BodyPart::Seat if direction == FlipDirection::Backward => BodyPart::Front,
                BodyPart::Head if direction == FlipDirection::Forward => BodyPart::Front,
                BodyPart::Head if direction == FlipDirection::Backward => BodyPart::Back,
                _ => BodyPart::Feet,
            }
        }
    }
}

#[derive(PartialEq, Clone, Savefile, Debug)]
struct Skill {
    flip: f32,
    from: BodyPart,
    to: BodyPart,
    twist: Vec<f32>,
    shape: Shape,
    direction: FlipDirection,
    edit_text: String,
}

fn fraction(num: f32) -> String {
    if num == 0.5 {
        return "half".to_owned();
    } else if num == 0.25 {
        return "quarter".to_owned();
    } else if num == 1.0 {
        return "full".to_owned();
    } else {
        let str = num
            .to_string()
            .replace(".5", " 1/2")
            .replace(".25", " 1/4")
            .replace(".75", " 3/4");
        if num - num.fract() < 0.0 {
            let str = str.replace("0", "");
        }
        return str;
    }
}

fn no_icon(ui: &mut egui::Ui, openness: f32, response: &egui::Response) {}

impl Skill {
    /// todo

    fn name(&self) -> String {
        if let Some(name) = SKILLS.get(&self.notation()) {
            return name.to_owned().to_owned();
        }

        let mut name: String = match self.flip.to_string().as_str() {
            "1" => "Single, ".to_owned(),
            "2" => "Double, ".to_owned(),
            "3" => "Triple, ".to_owned(),
            "4" => "Quad, ".to_owned(),
            _ => format!("{} flip, ", fraction(self.flip)),
        };
        name += match self.direction {
            FlipDirection::Forward => "Forward, ",
            FlipDirection::Backward => "Backward, ",
        };

        if self.twist.len() > 1 {
            name += format!(
                " {} {} {}",
                match self.twist[0].ceil() as i32 {
                    0 => "".to_owned(),
                    _ => format!("{} in,", fraction(self.twist[0])),
                },
                self.twist
                    .iter()
                    .skip(1)
                    .filter(|x| **x != 0.0)
                    .enumerate()
                    .map(|(i, x)| { fraction(*x) })
                    .collect::<Vec<String>>()
                    .join(" twist,"),
                match self.twist.last() {
                    Some(a) if a.ceil() != 0.0 => "",
                    _ => "out",
                }
            ).as_str();
        }
        if self.twist.len() == 1 {
            name += (
                match self.twist[0] {
                    0.0 => "".to_owned(),
                    _ => format!(" {} twist", fraction(self.twist[0])),
                }
            ).as_str();
        }
        if self.flip.fract() != 0.0 || self.from != BodyPart::Feet || self.to == BodyPart::Seat {
            name += format!(", from {} to {}", self.from.name(), self.to.name()).as_str();
        }
        name = name.to_owned();

        name += match self.shape {
            _ if self.flip == 0.0 => "",
            Shape::Straight => " (Straight)",
            Shape::Pike => " (Pike)",
            Shape::Tuck => " (Tuck)",
        };
        return name;
    }

    fn notation(&self) -> String {
        ((self.flip * 4.0) as u32).to_string() +
            &self.twist
                .iter()
                .map(|x| (x * 2.0) as u32)
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join("") +
            &(match self.shape {
                Shape::Straight => " /",
                Shape::Pike => " <",
                Shape::Tuck => " o",
            }) +
            (match self.direction {
                FlipDirection::Forward => " f",
                FlipDirection::Backward => "",
            }) +
            (match self.to {
                BodyPart::Seat => " -1",
                _ => "",
            })
    }
    fn diff(&self) -> f32 {
        let mut diff = 0.0;

        // +1.0 for each 1/4 flip, plus 0.1 for each 1/2 twist
        diff += self.flip * 0.4;
        diff += self.twist.iter().sum::<f32>() * 0.2;

        // +0.1 for each completed 360 somersault (bonus)
        diff += self.flip.floor() * 0.1;

        match self.shape {
            Shape::Straight | Shape::Pike => {
                if self.twist.iter().sum::<f32>() == 0.0 && self.flip >= 1.0 {
                    diff += 0.1;
                }
                if self.flip >= 2.0 {
                    diff += self.flip.floor() * 0.1;
                }
                if self.flip >= 3.0 {
                    diff += (self.flip - 3.0).floor() * 0.1;
                }
            }
            Shape::Tuck => {}
        }
        return (diff * 100.0).round() / 100.0;
    }
    fn from_notation(notation: String, from: BodyPart) -> Option<Skill> {
        let mut num_flips = 0;
        let shape = match (notation.contains("o"), notation.contains("<"), notation.contains("/")) {
            (true, false, false) => Shape::Tuck,
            (false, true, false) => Shape::Pike,
            (false, false, true) => Shape::Straight,
            _ => {
                return None;
            }
        };

        let forwards = notation.contains("f");
        let to_seat = notation.contains("-1");

        let notation = notation
            .replace("-1", "")
            .chars()
            .filter(|x| x.is_ascii_digit())
            .collect::<String>();

        let mut current_text = "".to_owned();
        for (i, c) in notation.chars().enumerate() {
            current_text.push(c);
            let potential_number_of_flips = current_text.parse::<i32>().unwrap_or(0);
            let remaining_chars = notation.len() - i;

            if
                potential_number_of_flips >
                (match (remaining_chars * 4).try_into() {
                    Ok(a) => a,
                    Err(e) => -1,
                })
            {
                break;
            }

            num_flips = potential_number_of_flips;
        }
        // if num_flips != (notation.length() - num_flips.to_string().length())/4.0 {
        //         println!("{} {} {}", potential_number_of_flips, remaining_chars, notation.len());
        //         return None;
        // }

        if notation.len() == 1 {
            num_flips = 0;
            current_text = "".to_owned();
        }

        let twist = notation
            .chars()
            .skip(current_text.len().max(1) - 1)
            .map(|x| (x.to_digit(10).unwrap() as f32) / 2.0)
            .collect::<Vec<f32>>();

        return Some(Skill {
            to: match to_seat {
                true => BodyPart::Seat,
                false =>
                    from.add(
                        ((num_flips as f32) / 4.0).fract(),
                        match forwards {
                            true => FlipDirection::Forward,
                            false => FlipDirection::Backward,
                        },
                        twist.iter().sum()
                    ),
            },
            direction: match forwards {
                true => FlipDirection::Forward,
                false => FlipDirection::Backward,
            },
            flip: (num_flips as f32) / 4.0,
            twist,
            from,
            edit_text: notation +
            (match &shape {
                Shape::Straight => " /",
                Shape::Pike => " <",
                Shape::Tuck => " o",
            }),
            shape,
        });
    }

    fn display(
        &mut self,
        egui_ctx: &egui::Context,
        ui: &mut egui::Ui,
        from: BodyPart,
        id: String
    ) -> BodyPart {
        let mut changed = false;
        ui.horizontal(|ui| {
            egui::CollapsingHeader
                ::new(self.name())
                .id_source(id)
                .icon(no_icon)
                .show(ui, |ui| {
                    ui.label("Notation                  ");
                    ui.shrink_width_to_current();
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.edit_text);
                        let mut valid = false;
                        if let Some(skill) = Skill::from_notation(self.edit_text.clone(), from) {
                            valid = true;
                            self.flip = skill.flip;
                            self.twist = skill.twist;
                            self.shape = skill.shape;
                            self.from = skill.from;
                            self.to = skill.to;
                            self.direction = skill.direction;
                        }
                        ui.checkbox(&mut valid, "valid").surrender_focus();
                        ui.add_space(10.0);
                        ui.hyperlink_to("FIG", "https://usagym.org/PDFs/Forms/T%26T/DD_TR.pdf")
                    });
                    ui.separator();
                    ui.label("Flip");
                    ui.horizontal(|ui| {
                        if self.flip >= 0.25 {
                            ui.small_button("-0.25")
                                .clicked()
                                .then(|| {
                                    self.flip -= 0.25;
                                    changed = true;
                                });
                        } else {
                            ui.label("-0.25");
                        }
                        if self.flip >= 1.0 {
                            ui.small_button("-1.0")
                                .clicked()
                                .then(|| {
                                    self.flip -= 1.0;
                                    changed = true;
                                });
                        } else {
                            ui.label("-1.0");
                        }
                        ui.selectable_label(true, format!("{:.2}", self.flip)).on_hover_text(
                            "Number of flips"
                        );
                        if self.flip <= 9.0 {
                            ui.small_button("+1.0")
                                .clicked()
                                .then(|| {
                                    self.flip += 1.0;
                                    changed = true;
                                });
                        } else {
                            ui.label("+1.0");
                        }
                        if self.flip <= 9.75 {
                            ui.small_button("+0.25")
                                .clicked()
                                .then(|| {
                                    self.flip += 0.25;
                                    changed = true;
                                });
                        } else {
                            ui.label("+0.25");
                        }
                    });
                    while self.twist.len() > (self.flip.ceil() as usize) {
                        self.twist.pop();
                    }
                    for i in 0..self.flip.ceil() as usize {
                        if self.twist.len() <= i {
                            self.twist.push(0.0);
                        }
                        ui.horizontal(|ui| {
                            ui.label(format!("{}.)", i + 1)).on_hover_text(
                                format!("Twists for flip {}", i + 1)
                            );
                            if self.flip >= 0.25 {
                                ui.small_button("-0.5")
                                    .clicked()
                                    .then(|| {
                                        self.twist[i] -= 0.5;
                                        changed = true;
                                    });
                            } else {
                                ui.label("-0.5");
                            }
                            ui.selectable_label(
                                true,
                                format!("{:.1}", self.twist[i])
                            ).on_hover_text(format!("number of twists in flip no. {}", i + 1));
                            if self.flip <= 9.75 {
                                ui.small_button("+0.5")
                                    .clicked()
                                    .then(|| {
                                        self.twist[i] += 0.5;
                                        changed = true;
                                    });
                            } else {
                                ui.label("+0.5");
                            }
                        });
                    }

                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.shape, Shape::Tuck, "tuck")
                            .changed()
                            .then(|| {
                                changed = true;
                            });
                        ui.radio_value(&mut self.shape, Shape::Pike, "pike")
                            .changed()
                            .then(|| {
                                changed = true;
                            });
                        ui.radio_value(&mut self.shape, Shape::Straight, "straight")
                            .changed()
                            .then(|| {
                                changed = true;
                            });
                    });
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.direction, FlipDirection::Forward, "forward")
                            .changed()
                            .then(|| {
                                changed = true;
                            });
                        ui.radio_value(&mut self.direction, FlipDirection::Backward, "backward")
                            .changed()
                            .then(|| {
                                changed = true;
                            });
                    });
                    ui.horizontal(|ui| {
                        let new_body = match self.to {
                            BodyPart::Seat => BodyPart::Feet,
                            BodyPart::Back => BodyPart::Back,
                            BodyPart::Feet => BodyPart::Feet,
                            BodyPart::Front => BodyPart::Front,
                            BodyPart::Head => BodyPart::Head,
                        };
                        ui.radio_value(&mut self.to, BodyPart::Seat, "To Seat")
                            .changed()
                            .then(|| {
                                changed = true;
                            });
                        ui.radio_value(&mut self.to, new_body, format!("To {}", new_body.name()))
                            .changed()
                            .then(|| {
                                changed = true;
                            });
                    });
                })
                .fully_open()
                .then(|| ui.separator());

            ui.add_sized(
                ui.available_size(),
                egui::Label::new(format!("Diff: {}", self.diff()))
            ).on_hover_ui(|ui| {
                ui.label("Difficulty");
                ui.hyperlink("https://usagym.org/PDFs/Forms/T%26T/DD_TR.pdf");
            });
        });
        if changed {
            self.edit_text = self.notation();
        }
        return self.to;
    }
}

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
    open: bool,
}

struct Video {
    path: String,
    open: bool,
    ffmpeg_version: (String, String),
    textures: Vec<(Texture2D, f32)>,
    current_frame: usize,
    show_video: bool,
    kill: bool,
    rect: [f32; 4],
    timestamps: bool,
    drag_one: usize,
}

impl Video {
    fn load_textures(&mut self) -> Result<(), ffmpeg_sidecar::error::Error> {
        // let mut textures = Vec::new();
        match nfd2::open_file_dialog(None, None) {
            Ok(Response::Okay(file_path)) => {
                self.path = file_path.display().to_string();
                println!("path: {}", self.path);
            }
            _ => {
                println!("no file selected");
            }
        }

        FfmpegCommand::new()
            .duration("30")
            .input(&self.path)
            .hide_banner()
            .filter("fps=fps=24")
            .args(match self.timestamps {true => ["-vf","scale=720:-1, drawtext=fontsize=50:fontcolor=GreenYellow:text='%{e\\:t}':x=(w-text_w):y=(h-text_h)"], false => ["",""]})
            .args(["-f", "rawvideo", "-pix_fmt", "rgba", "-"])
            // .rawvideo()
            .spawn()?
            .iter()?
            .for_each(|event: FfmpegEvent| {
                match event {
                    FfmpegEvent::OutputFrame(frame) => {
                        self.textures.push((
                            Texture2D::from_rgba8(
                                frame.width as u16,
                                frame.height as u16,
                                &frame.data
                            ),
                            frame.timestamp,
                        ));
                    }
                    FfmpegEvent::Progress(progress) => {
                        eprintln!("Current speed: {}x", progress.speed);
                    }
                    FfmpegEvent::Log(_level, msg) => {
                        eprintln!("[ffmpeg] {}", msg);
                    }
                    _ => {}
                }
            });

        println!("Finished {} frames", self.textures.len());
        Ok(())
    }

    fn display(&mut self, egui_ctx: &egui::Context) {
        egui::Window
            ::new("Video")
            .scroll2([true, false])
            .min_height(match self.textures.get(self.current_frame) {
                Some(a) => a.0.height() + 10.0,
                _ => 0.0,
            })
            .show(egui_ctx, |ui| {
                // Show the image:

                ui.separator();
                match self.textures.len() {
                    0 => {
                        self.show_video = false;
                        if
                            ui
                                .add_sized([ui.available_width(), ui.available_width()*0.65], egui::Button::new("download file"))
                                .clicked()
                        {
                            self.load_textures();
                        }
                    }
                    _ => {
                        if self.current_frame >= self.textures.len() {
                            self.current_frame = 0;
                        }
                        let ratio =
                            (self.textures[self.current_frame].0.height() as f32) /
                            (self.textures[self.current_frame].0.width() as f32);
                        let r = ui.add_sized(
                            [ui.available_width(), ui.available_width() * ratio],
                            egui::Label::new(format!("frame: {}", self.current_frame))
                        ).rect;
                        self.rect = [r.min.x, r.min.y, r.max.x, r.max.y];
                        ui.add(
                            egui::Slider
                                ::new(&mut self.drag_one, 0..=self.textures.len() - 1)
                                .clamp_to_range(true)
                                .text("frame")
                                .drag_value_speed(0.5)
                        );

                        let bar = ui.add(
                            egui::ProgressBar::new(
                                (self.current_frame as f32) / (self.textures.len() as f32)
                            )
                        );
                        match bar.hover_pos() {
                            Some(pos) => {
                                self.current_frame = (
                                    (((pos.x - bar.rect.left()) / bar.rect.width()) *
                                        (self.textures.len() as f32)) as usize
                                ).clamp(0, self.textures.len() - 1);
                                if ui.input(|i| i.pointer.any_click()) {
                                    self.drag_one = self.current_frame;
                                }
                            }
                            _ => {
                                self.current_frame = self.drag_one.clamp(
                                    0,
                                    self.textures.len() - 1
                                );
                            }
                        }

                        ui.separator();
                        // number input
                        ui.label(format!("Time: {}", self.textures[self.current_frame].1));
                        ui.checkbox(&mut self.show_video, "Render Video");
                    }
                }
                ui.separator();
                ui.small_button("Close Window")
                    .clicked()
                    .then(|| {
                        self.open = false;
                    });
                ui.small_button("Kill Window")
                    .clicked()
                    .then(|| {
                        self.kill = true;
                    });
            });
    }
    fn new() -> Video {
        Video {
            path: String::from(""),
            open: true,
            ffmpeg_version: (
                match ffmpeg_sidecar::download::check_latest_version() {
                    Ok(version) => version,
                    Err(a) => format!("{a:?}"),
                },
                match ffmpeg_sidecar::version::ffmpeg_version() {
                    Ok(version) => version,
                    Err(a) => format!("{a:?}"),
                },
            ),
            textures: Vec::new(),
            current_frame: 0,
            show_video: true,
            kill: false,
            rect: [0.0, 0.0, 0.0, 0.0],
            drag_one: 0,
            timestamps: true,
        }
    }
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
                                "Total Difficulty: {}",
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
            id: UNIX_EPOCH.elapsed().unwrap().as_secs().to_string(),
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
}

impl Data {
    fn render(&mut self, egui_ctx: &egui::Context) {
        for r in self.routines.iter_mut() {
            r.display(&egui_ctx);
        }
    }

    fn save(&self) {
        match fs::create_dir_all("./Data/routines") {
            Ok(_) => {}
            Err(e) => {
                error!("Error creating directory: {}", e);
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
        for file in (
            match fs::read_dir("./Data/routines") {
                Ok(file) => file,
                Err(e) => {
                    error!("Error reading directory: {}", e);
                    return;
                }
            }
        ) {
            self.routines.clear();
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
            let routine: Routine = match savefile::load_file(path, 1) {
                Ok(routine) => routine,
                Err(e) => {
                    error!("Error loading file: {}", e);
                    return;
                }
            };
            self.routines.push(routine);
        }
    }
}

#[derive(Debug, Clone, Savefile)]
enum Panel {
    Routine,
    TOF,
    Execution,
    Deductions,
}

impl Default for Panel {
    fn default() -> Self {
        Panel::Routine
    }
}

fn none_routine() -> Option<Routine> {
    None
}

#[derive(Debug, Clone, Savefile)]
struct RecordedRoutine {
    #[savefile_default_fn = "none_routine"]
    #[savefile_ignore]
    routine: Option<Routine>,
    #[savefile_ignore]
    panel: Panel,
    routine_id: String,
    execution_1: [f32;10],
}

#[macroquad::main("Trampoline thing")]
async fn main() {
    let mut now = Instant::now();
    // get text input from user
    let mut data = Data {
        routines: vec![],
        theme: WindowTheme::Light,
    };

    // let mut

    ffmpeg_sidecar::download::auto_download().unwrap();

    data.load_files();
    egui_macroquad::ui(|egui_ctx| {
        data.theme.set_theme(egui_ctx);
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
                ui.heading("Routines");
                ui.separator();
                ui.button("New Routine")
                    .clicked()
                    .then(|| {
                        data.routines.push(Routine::blank());
                    });
                ui.menu_button("edit routine", |ui| {
                    for r in data.routines.iter_mut() {
                        let toggle = !r.open;
                        ui.selectable_value(&mut r.open, toggle, &r.name);
                    }
                });
                ui.heading("Video");
                ui.separator();
                ui.button("load video")
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
                ui.button("Update/download ffmpeg")
                    .clicked()
                    .then(|| {
                        ffmpeg_sidecar::download::auto_download().unwrap();
                    });
                ui.heading("Files");
                ui.separator();
                ui.button("Save")
                    .clicked()
                    .then(|| {
                        data.save();
                    });
                ui.heading("Settings");
                ui.separator();
                ui.menu_button("UI Style", |ui| {
                    for s in WindowTheme::iter() {
                        ui.selectable_value(&mut data.theme, s, &format!("{:?}", s));
                    }
                })
                    .response.clicked()
                    .then(|| {
                        data.theme.set_theme(egui_ctx);
                    });
            });
            for i in videos.iter_mut() {
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

        videos.retain(|x| !x.kill);

        egui_macroquad::draw();
        for v in &videos {
            if v.show_video && v.open {
                let index = clamp(v.current_frame, 0, v.textures.len() - 1);
                let frame = &v.textures[index];
                draw_texture_ex(frame.0, v.rect[0], v.rect[1], WHITE, DrawTextureParams {
                    dest_size: Some(vec2(v.rect[2] - v.rect[0], v.rect[3] - v.rect[1])),
                    ..Default::default()
                });
            }
        }

        next_frame().await;
    }
}