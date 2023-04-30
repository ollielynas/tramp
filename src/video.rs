use std::ops::RangeInclusive;

use clipboard::{ClipboardContext, ClipboardProvider};
use egui::plot::{HLine, Plot, PlotPoint, PlotPoints, Points, VLine, Line, Polygon};
use ffmpeg_sidecar::{command::FfmpegCommand, event::FfmpegEvent};
use macroquad::texture::Texture2D;
use nfd2::Response;



pub struct Video {
    pub path: String,
    pub open: bool,
    pub textures: Vec<(Texture2D, f32)>,
    pub current_frame: usize,
    pub show_video: bool,
    pub kill: bool,
    pub rect: [f32; 4],
    pub timestamps: bool,
    pub drag_one: usize,
    pub points: Vec<f32>,
    add_points: bool,
    
}

impl Video {
    pub fn load_textures(&mut self) -> Result<(), ffmpeg_sidecar::error::Error> {
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
            .filter(format!("fps=fps={}", 48.0).as_str())
            .args(match self.timestamps {true => ["-vf","scale=w='if(gte(iw,ih),720,-1)':h='if(lt(iw,ih),720,-1)', drawtext=fontsize=50:fontcolor=GreenYellow:text='%{e\\:t}':x=(w-text_w):y=(h-text_h)"], false => ["",""]})
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

    pub fn display(&mut self, egui_ctx: &egui::Context) {
        let framerate = match self.textures.last() {Some(a) => {a.1}, None => 0.0}/self.textures.len() as f32;
        egui::Window::new("Video")
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
                        if ui
                            .add_sized(
                                [ui.available_width(), ui.available_width() * 0.65],
                                egui::Button::new("download file"),
                            )
                            .clicked()
                        {
                            self.load_textures();
                        }
                    }
                    _ => {
                        if self.current_frame >= self.textures.len() {
                            self.current_frame = 0;
                        }
                        let ratio = (self.textures[self.current_frame].0.height() as f32)
                            / (self.textures[self.current_frame].0.width() as f32);
                        let r = ui
                            .add_sized(
                                [ui.available_width(), ui.available_width() * ratio],
                                egui::Label::new(format!("frame: {}", self.current_frame)),
                            )
                            .rect;
                        self.rect = [r.min.x, r.min.y, r.max.x, r.max.y];
                        ui.add(
                            egui::Slider::new(&mut self.drag_one, 0..=self.textures.len() - 1)
                                .clamp_to_range(true)
                                .text("frame")
                                .drag_value_speed(0.5),
                        );

                        fn point_label(p: f64, _range: &RangeInclusive<f64>) -> String {
                            format!("{p}sec")
                        }

                        fn label_formatter(name: &str, p: &PlotPoint) -> String {
                            format!("{} {:.2}sec", name, p.x)
                        }

                        let mut ToF: f32 = 0.0;

                        self.points.sort_by(|a, b| a.partial_cmp(b).unwrap());
                        let bar = Plot::new("my_plot")
                            .show_background(true)
                            .allow_drag(false)
                            .allow_zoom(false)
                            .allow_scroll(false)
                            .include_x(0.0)
                            .include_y(0.0)
                            .include_y(10.0)
                            .show_y(false)
                            .label_formatter(label_formatter)
                            .x_axis_formatter(point_label)
                            .view_aspect(1200.0)
                            .include_x(self.textures[self.textures.len()-1].1)
                            .show(ui, |plot_ui| {

                                for (j, i) in self.points.iter().enumerate() {
                                    match j % 2 {
                                        0 => {
                                            if j != 0 {
                                                plot_ui.polygon(
                                                    Polygon::new(vec![
                                                        [self.points[j-1] as f64,1.0] as _,[*i as f64,1.0] as _,
                                                        [*i as f64,0.0] as _,[self.points[j-1] as f64,0.0] as _
                                                    ]).fill_alpha(0.5).color(egui::Color32::from_rgb(255, 0, 0)).width(0.0)
                                                    
                                                );
                                                
                                            }
                                            plot_ui.vline(
                                                VLine::new(*i)
                                                    .color(egui::Color32::from_rgb(0, 255, 0))
                                                    .name("start jump"),
                                            );

                                        }
                                        _ => {
                                            plot_ui.polygon(
                                                Polygon::new(vec![
                                                    [self.points[j-1] as f64,2.0] as _,[*i as f64,2.0] as _,
                                                    [*i as f64,1.0] as _,[self.points[j-1] as f64,1.0] as _
                                                ]).fill_alpha(0.5).color(egui::Color32::from_rgb(0, 255, 0)).width(0.0)
                                                
                                            );
                                            plot_ui.vline(
                                                VLine::new(*i)
                                                    .color(egui::Color32::from_rgb(255, 0, 0))
                                                    .name("end jump"),
                                            );
                                            ToF += *i - self.points[j-1];
                                                
                                        }
                                    }
                                }
                                
                                plot_ui.vline(
                                    VLine::new(self.drag_one as f32 / framerate)
                                        .highlight(self.current_frame == self.drag_one)
                                        .color(egui::Color32::from_rgb(0, 0, 255)),
                                );
                            })
                            .response;


                        ui.horizontal(|ui| {
                            if ui
                                .selectable_label(!self.add_points, "move player")
                                .clicked()
                            {
                                self.add_points = false;
                            };
                            if ui.selectable_label(self.add_points, "add points").clicked() {
                                self.add_points = true;
                            };
                        });

                        match bar.hover_pos() {
                            Some(pos) => {
                                ui.separator();
                                self.current_frame = ((((pos.x - bar.rect.left())
                                    / bar.rect.width())
                                    * (self.textures.len() as f32))
                                    as usize)
                                    .clamp(0, self.textures.len() - 1);
                                if self.add_points {
                                    if match self.points.iter().map(|p| (self.textures[self.current_frame].1).abs()).fold(f32::MAX, |acc, e| acc.min(e)) {a if a != f32::MAX => a < 5.0, _ => false}  {
                                        ui.label("remove point");
                                        egui_ctx.output_mut(|o| {
                                            o.cursor_icon = egui::CursorIcon::NotAllowed;
                                        });
                                        if ui.input(|i| i.pointer.any_click()) {

                                            self.points.retain(|p| ((self.textures[self.current_frame].1) as i32).abs() > 5);
                                        }
                                    } else {
                                        if ui.input(|i| i.pointer.any_click()) {
                                            self.points.push(self.textures[self.current_frame].1);
                                            
                                        }
                                    }
                                } else {
                                    ui.label("add point");
                                    if ui.input(|i| i.pointer.any_click()) {
                                        self.drag_one = self.current_frame;
                                    }
                                }
                            }
                            _ => {
                                self.current_frame =
                                    self.drag_one.clamp(0, self.textures.len() - 1);
                            }
                        }
                        ui.separator();
                        ui.label(format!("Number of skills: {}", (self.points.len() as f32/2.0).floor()));
                        ui.horizontal(|ui| {
                            ui.label(format!("ToF: {}", ToF));
                            if ui.small_button( egui_phosphor::COPY).on_hover_text("Copy to clipboard").clicked() {
                                egui_ctx.output_mut(|o| {
                                            o.copied_text = format!("{}", ToF);
                                        });
                            }
                        });

                        ui.separator();
                        // number input
                        ui.label(format!("Time: {}", self.textures[self.current_frame].1));
                        ui.checkbox(&mut self.show_video, "Render Video");
                    }
                }
                
                ui.separator();
                ui.small_button("Close Window").clicked().then(|| {
                    self.open = false;
                });
                ui.small_button("Kill Window").clicked().then(|| {
                    self.kill = true;
                });
            });
    }
    pub fn new() -> Video {
        Video {
            add_points: false,
            path: String::from(""),
            open: true,
            textures: Vec::new(),
            current_frame: 0,
            show_video: true,
            kill: false,
            rect: [0.0, 0.0, 0.0, 0.0],
            drag_one: 0,
            timestamps: true,
            points: Vec::new(),
        }
    }
}
