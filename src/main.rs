use egui::{widgets};
use macroquad::prelude::*;
use std::io;

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
}
#[derive(PartialEq, Eq, Hash, Clone, Copy)]
enum FlipDirection {
    Forward,
    Backward,
}

impl BodyPart {
    fn name(&self) -> String {
        match self {
            BodyPart::Feet => "feet",
            BodyPart::Front => "front",
            BodyPart::Back => "back",
            BodyPart::Head => "head",

        }
        .to_owned()
    }
    fn add(&self, amount: f32, direction:FlipDirection, total_twist: f32) -> BodyPart {
        let direction = match (direction, (total_twist.fract()*10.0) as i32) {
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
            }
        } else if amount == 0.25 {
            match &self {
                BodyPart::Back | BodyPart::Front => BodyPart::Feet,
                BodyPart::Feet if direction == FlipDirection::Forward => BodyPart::Front,
                BodyPart::Feet if direction == FlipDirection::Backward => BodyPart::Back,
                BodyPart::Head if direction == FlipDirection::Forward => BodyPart::Back,
                BodyPart::Head if direction == FlipDirection::Backward => BodyPart::Front,
                _ => BodyPart::Feet,
                }
        }else {
            match &self {
                BodyPart::Back | BodyPart::Front => BodyPart::Feet,
                BodyPart::Feet if direction == FlipDirection::Forward => BodyPart::Back,
                BodyPart::Feet if direction == FlipDirection::Backward => BodyPart::Front,
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
    twist: Vec<f32>,
    shape: Shape,
    direction: FlipDirection,
    edit_text: String,
}

fn fraction(num: f32) -> String {

    if num == 0.5 {
        return "half".to_owned();
    }
    else if num == 0.25 {
        return "quarter".to_owned();
    }
    else if num == 1.0 {
        return "full".to_owned();
    }else {
    let str = num.to_string()
        .replace(".5", "1/2")
        .replace(".25", "1/4")
        .replace(".75", "3/4");
    if num - num.fract() < 0.0 {
        let str = str.replace("0", "");
    }
    return str
    }
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
            if self.twist.iter().sum::<f32>() == 0.0 {
                return match self.shape {
                    Shape::Straight => "Straight Jump",
                    Shape::Pike => "Pike Jump",
                    Shape::Tuck => "Tuck Jump",
                }.to_owned()
            }
        }
                if self.twist.len() > 1 {
                    name += format!(
                        " {} {} {}",
                        match self.twist[0] {0.0 => "".to_owned(), _ => format!("{} in,", fraction(self.twist[0]))},
                        self.twist
                            .iter().skip(1).filter(|x| **x != 0.0).enumerate()
                            .map(|(i, x)| {
                                fraction(*x)
                            })
                            .collect::<Vec<String>>()
                            .join(" twist,"),
                        match self.twist.last()  {Some(0.0) => "", _ => "out"}
                        
                    ).as_str();
                }
                if self.twist.len() == 1 {
                    name += match self.twist[0] {0.0 => "".to_owned(), _ => format!(" {} twist", fraction(self.twist[0]))}.as_str();
                }
                if self.flip.fract() != 0.0 || self.from != BodyPart::Feet {
                    name += format!(", from {} to {}",self.from.name(), self.from.add(self.flip.fract(), self.direction, self.twist.iter().sum()).name()).as_str();
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
        return (diff*100.0).round()/100.0;
    }
    fn from_notation(notation: String) -> Option<Skill> {
        let mut num_flips = 0;
        let shape = match (notation.contains("o"), notation.contains("<"), notation.contains("/")) {
            (true, false, false) => Shape::Tuck,
            (false, true, false) => Shape::Pike,
            (false, false, true) => Shape::Straight,
            _ => {
                return None;
            }
        };
        let notation = notation
            .chars()
            .filter(|x| x.is_ascii_digit())
            .collect::<String>();
        
        let mut current_text = "".to_owned();
        for (i, c) in notation.chars().enumerate() {
            current_text.push(c);
            let potential_number_of_flips = current_text.parse::<i32>().unwrap_or(0);
            let remaining_chars = notation.len() - i;

            if potential_number_of_flips > (remaining_chars * 4).try_into().unwrap() {
                break;
            }
            num_flips = potential_number_of_flips;
        }

        if notation.len() == 1 {
            num_flips = 0;
            current_text = "".to_owned();
        }

        let twist = notation
            .chars()
            .skip((current_text.len().max(1)-1))
            .map(|x| (x.to_digit(10).unwrap() as f32) / 2.0)
            .collect::<Vec<f32>>();

        return Some(Skill {
            direction: FlipDirection::Forward,
            flip: (num_flips as f32) / 4.0,
            twist,
            from: BodyPart::Feet,
            edit_text: notation + match &shape {
                Shape::Straight => " /",
                Shape::Pike => " <",
                Shape::Tuck => " o",
            },
            shape,
        });
    }

    fn display(&mut self, egui_ctx: &egui::Context, ui: &mut egui::Ui) {
            ui.horizontal(|ui| {
                ui.collapsing("", |ui| {
                    ui.label("Notation                  ");
                    ui.shrink_width_to_current();
                    if ui.text_edit_singleline(&mut self.edit_text).changed() {
                    if let Some(skill) = Skill::from_notation(self.edit_text.clone()) {
                        self.flip = skill.flip;
                        self.twist = skill.twist;
                        self.shape = skill.shape;
                    }
                    }
                    ui.label("Shape");
                    ui.radio_value(&mut self.shape, Shape::Tuck, "tuck").changed();
                    ui.radio_value(&mut self.shape, Shape::Pike, "pike").changed();
                    ui.radio_value(&mut self.shape, Shape::Straight, "straight").changed();
                    ui.label("FlipDirection");
                    ui.radio_value(&mut self.direction, FlipDirection::Forward, "forward").changed();
                    ui.radio_value(&mut self.direction, FlipDirection::Backward, "backward").changed();
                });
                ui.label(self.name());
                ui.label(format!("Diff: {}", self.diff())).on_hover_text("Difficulty");
            });
            
    }
}


struct Routine {
    skills: [Skill;10],

}

impl Routine {
    fn display(&mut self, egui_ctx: &egui::Context,) {
        egui::Window::new("Routine").show(egui_ctx, |ui| {
            for (i, skill) in self.skills.iter_mut().enumerate() {
                ui.push_id(i, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}: ", i+1));
                        skill.display(egui_ctx, ui);
                    });
                    });
            }
        });
    }
    fn blank() -> Routine {
        Routine {
            skills: [
                Skill::from_notation("0 o".to_owned()).unwrap(),
                Skill::from_notation("0 o".to_owned()).unwrap(),
                Skill::from_notation("0 o".to_owned()).unwrap(),
                Skill::from_notation("0 o".to_owned()).unwrap(),
                Skill::from_notation("0 o".to_owned()).unwrap(),
                Skill::from_notation("0 o".to_owned()).unwrap(),
                Skill::from_notation("0 o".to_owned()).unwrap(),
                Skill::from_notation("0 o".to_owned()).unwrap(),
                Skill::from_notation("0 o".to_owned()).unwrap(),
                Skill::from_notation("0 o".to_owned()).unwrap(),
            ],
        }
    }
}

#[macroquad::main("Trampoline thing")]
async fn main() {
    // get text input from user
    let mut routine = Routine::blank();
    loop {
        clear_background(WHITE);

        // Process keys, mouse etc.
        egui_macroquad::ui(|egui_ctx| {
            
            egui_ctx.set_visuals(egui::Visuals::light());
            routine.display(egui_ctx);
        });

        // Draw things before egui

        egui_macroquad::draw();

        // Draw things after egui

        next_frame().await;
    }
}