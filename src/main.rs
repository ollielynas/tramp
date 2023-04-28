use egui::{ widgets, Id };
use macroquad::{ prelude::*, time };
use std::{ io, ops::RangeInclusive };

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum Shape {
    Straight,
    Pike,
    Tuck,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum BodyPart {
    Feet,
    Front,
    Back,
    Head,
    Seat,
}
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
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

fn circle_icon(ui: &mut egui::Ui, openness: f32, response: &egui::Response) {
    // let stroke = ui.style().interact(&response).fg_stroke;
    // let radius = egui::lerp(2.0..=3.0, openness);
    // ui.painter().circle_filled(response.rect.center(), radius, stroke.color);
}

impl Skill {
    /// todo

    fn name(&self) -> String {
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
        if self.flip == 0.0 {
            name = "".to_owned();
            if self.twist.iter().sum::<f32>() == 0.0 && self.to != BodyPart::Seat {
                return (
                    match self.shape {
                        Shape::Straight => "Straight Jump",
                        Shape::Pike => "Pike Jump",
                        Shape::Tuck => "Tuck Jump",
                    }
                ).to_owned();
            }
        }
        if self.twist.len() > 1 {
            name += format!(
                " {} {} {}",
                match self.twist[0] {
                    0.0 => "".to_owned(),
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
                    Some(0.0) => "",
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
        println!("{diff}");
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
                .icon(circle_icon)
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
                .then(|| { ui.separator() });

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

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    Edit,
    Info,
    DragAndDrop,
    Metadata,
}

struct Routine {
    skills: [Skill; 10],
    name: String,
    current_tab: Tab,
    id: String,
    open: bool,
}

impl Routine {
    fn display(&mut self, egui_ctx: &egui::Context) {
        egui::Window
            ::new(format!("Routine: {}", self.name))
            .id(Id::new(&self.id))
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
                    Tab::DragAndDrop => {}
                    Tab::Metadata => {}
                }
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
            id: time::get_time().to_string(),
            open: true,
        }
    }
}

struct Data {
    routines: Vec<Routine>,
}

impl Data {
    fn render(&mut self, egui_ctx: &egui::Context) {
        for r in self.routines.iter_mut() {
            r.display(&egui_ctx);
        }
    }
}

#[macroquad::main("Trampoline thing")]
async fn main() {
    egui_macroquad::ui(|egui_ctx| {
        egui_ctx.set_visuals(egui::Visuals::light());
    });
    // get text input from user
    let mut data = Data {
        routines: vec![Routine::blank()],
    };
    loop {
        clear_background(WHITE);
        // Process keys, mouse etc.
        egui_macroquad::ui(|egui_ctx| {
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
                    
                
            });
            
            data.render(&egui_ctx);
        });

        egui_macroquad::draw();
        next_frame().await;
    }
}