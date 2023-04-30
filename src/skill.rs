use crate::common_skills::SKILLS;




pub fn fraction(num: f32) -> String {
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

#[derive(PartialEq, Eq, Hash, Clone, Copy, Savefile, Debug)]
pub enum Shape {
    Straight,
    Pike,
    Tuck,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Savefile, Debug)]
pub enum BodyPart {
    Feet,
    Front,
    Back,
    Head,
    Seat,
}
#[derive(PartialEq, Eq, Hash, Clone, Copy, Savefile, Debug)]
pub enum FlipDirection {
    Forward,
    Backward,
}

impl BodyPart {
    pub fn name(&self) -> String {
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
    pub fn add(&self, amount: f32, direction: FlipDirection, total_twist: f32) -> BodyPart {
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
pub struct Skill {
    pub flip: f32,
    pub from: BodyPart,
    pub to: BodyPart,
    pub twist: Vec<f32>,
    pub shape: Shape,
    pub direction: FlipDirection,
    pub edit_text: String,
}


pub fn no_icon(ui: &mut egui::Ui, openness: f32, response: &egui::Response) {}

impl Skill {
    /// todo

    pub fn name(&self) -> String {
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

    pub fn notation(&self) -> String {
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
    pub fn diff(&self) -> f32 {
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
    pub fn from_notation(notation: String, from: BodyPart) -> Option<Skill> {
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

    pub fn display(
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
