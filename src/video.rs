use std::{ ops::RangeInclusive, time::UNIX_EPOCH };
use egui::{plot::{ HLine, Plot, PlotPoint, Polygon, VLine }, PointerButton};
use ffmpeg_sidecar::{ command::FfmpegCommand, event::{ FfmpegEvent, OutputVideoFrame } };
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
    start_from: f32,
    id: String,
    thread: Vec<std::thread::JoinHandle<Vec<OutputVideoFrame>>>,
    interpolate: bool,
    pub full_size: bool,
    skill_tof: [f32;10],
}

impl Video {
    fn load_textures(
        path2: Option<String>,
        timestamps: bool,
        start_from: f32,
        frame_interpolation: bool
    ) -> Result<Vec<OutputVideoFrame>, ffmpeg_sidecar::error::Error> {
        let mut textures = Vec::new();
        let path;
        if path2.is_none() {
            path = match nfd2::open_file_dialog(None, None) {
                Ok(Response::Okay(file_path)) => { file_path.display().to_string() }
                _ => { "".to_string() }
            };
        } else {
            path = path2.unwrap();
        }
        FfmpegCommand::new()
            .duration("30")
            .args(["-ss", start_from.to_string().as_str()])
            .input(path)
            .hide_banner()
            .filter(
                (
                    match frame_interpolation {
                        true => format!("minterpolate=fps={}:mi_mode=mci", 48.0),
                        false => format!("fps=fps={}", 48.0),
                    }
                ).as_str()
            )
            .args(match timestamps {
                true =>
                    [
                        "-vf",
                        "scale=w='if(gte(iw,ih),720,-1)':h='if(lt(iw,ih),720,-1)', drawtext=fontsize=50:fontcolor=GreenYellow:text='%{e\\:t}':x=(w-text_w):y=(h-text_h)",
                    ],
                false => ["-vf", "scale=w='if(gte(iw,ih),720,-1)':h='if(lt(iw,ih),720,-1)'"],
            })
            .args(["-f", "rawvideo", "-pix_fmt", "rgba", "-"])
            // .rawvideo()
            .spawn()?
            .iter()?
            .for_each(|event: FfmpegEvent| {
                match event {
                    FfmpegEvent::OutputFrame(frame) => {
                        textures.push(frame);
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

        println!("Finished {} frames", textures.len());
        Ok(textures)
    }

    pub fn display(&mut self, egui_ctx: &egui::Context) {
        
        let framerate =
            (match self.textures.last() {
                Some(a) => { a.1 }
                None => 0.0,
            }) / (self.textures.len() as f32);
        egui::Window
            ::new("Video")
            .id(egui::Id::new(self.id.clone()))
            .scroll2([true, false])
            .min_height(match self.textures.get(self.current_frame) {
                Some(a) => a.0.height() + 10.0,
                _ => 0.0,
            })
            .show(egui_ctx, |ui| {

                self.full_size = true;
                // Show the image:
                if self.thread.len() != 0 {
                    ui.heading("Loading...");
                    if self.thread[self.thread.len() - 1].is_finished() {
                        match self.thread.pop().unwrap().join() {
                            Ok(a) => {
                                self.textures = a
                                    .iter()
                                    .map(|frame| (
                                        Texture2D::from_rgba8(
                                            frame.width as u16,
                                            frame.height as u16,
                                            &frame.data
                                        ),
                                        frame.timestamp,
                                    ))
                                    .collect();
                                self.show_video = true;
                            }
                            Err(_) => {}
                        }
                    }
                    return;
                }

                ui.separator();
                match self.textures.len() {
                    0 => {

                        let mut hovered_files = vec![];
                        let mut droped_file: Option<String> = None;
                        egui_ctx.input(|i| {
                            hovered_files = i.raw.hovered_files
                                .iter()
                                .map(|file| {
                                    match &file.path {
                                        Some(a) => a.display().to_string(),
                                        None => "".to_string(),
                                    }
                                })
                                .collect();
                            for file in i.raw.dropped_files.iter() {
                                match &file.path {
                                    Some(a) => {
                                        droped_file = Some(a.display().to_string());
                                    }
                                    None => {}
                                }
                            }
                        });
                        let file_input=
                            ui
                                .add_sized(
                                    [ui.available_width(), ui.available_width() * 0.65],
                                    egui::Button::new(
                                        format!(
                                            "{} upload file\n{} Right Click to Paste Path From Clipboard\n{} Drag and drop\n{}",
                                            egui_phosphor::UPLOAD,
                                            egui_phosphor::CLIPBOARD_TEXT,
                                            egui_phosphor::HAND,
                                            match hovered_files.len() {
                                                0 => "".to_owned(),
                                                _ => {
                                                    let string = format!(
                                                        "\n {} ",
                                                        egui_phosphor::FILE_VIDEO
                                                    );
                                                    hovered_files.join(&string)
                                                }
                                            }
                                        )
                                    )
                                );
                                
                                
                        if file_input.clicked() || file_input.clicked_by(PointerButton::Secondary)  {
                            let mut path = None;
                            if file_input.clicked_by(PointerButton::Secondary) {
                                println!("Right Clicked");
                                egui_ctx.output_mut(|o| {
                                    path =  Some(o.copied_text.clone());
                                });
                            }
                            let t = self.timestamps;
                            let start = self.start_from;
                            let inter = self.interpolate;
                            self.thread.push(
                                std::thread::spawn(move || {
                                    Self::load_textures(path, t, start, inter).unwrap()
                                })
                            );
                        }
                        
                        ui.horizontal(|ui| {
                            ui.label(format!("{} start from: ", egui_phosphor::SKIP_FORWARD));
                            ui.add(
                                egui::DragValue
                                    ::new(&mut self.start_from)
                                    .speed(0.1)
                                    .clamp_range(0.0..=100.0)
                                    .suffix("sec")
                            );
                        });
                        ui.checkbox(&mut self.timestamps, format!("{} timestamps", egui_phosphor::WATCH));
                        ui.checkbox(&mut self.interpolate, format!("{} interpolate frames", egui_phosphor::INTERSECT_SQUARE));
                    }
                    _ => {
                        let frame_data = self.textures[self.current_frame].0;
                        let timestamp = self.textures[self.current_frame].1;

                        if self.current_frame >= self.textures.len() {
                            self.current_frame = 0;
                        }
                        let ratio =
                            (frame_data.height() as f32) /
                            (frame_data.width() as f32);
                        

                        let r = ui.add_sized([ui.available_width(), ui.available_width()*ratio], egui::Label::new(
                            match self.show_video {
                                true => {
                                    "".to_owned()
                                }
                                false => {
                                    format!("{} {}x{}\n",
                                    egui_phosphor::FRAME_CORNERS,
                                    frame_data.width(),
                                    frame_data.height()
                                )
                                }
                            }
                        ));

                        self.rect = [r.rect.left() , r.rect.top(), r.rect.width(), r.rect.height()];

                        fn point_label(p: f64, _range: &RangeInclusive<f64>) -> String {
                            format!("{p}sec")
                        }

                        fn label_formatter(name: &str, p: &PlotPoint) -> String {
                            format!("{} {:.2}sec", name, p.x)
                        }

                        let mut ToF: f32 = 0.0;
                        self.skill_tof = [0.0;10];
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
                            .include_x(self.textures[self.textures.len() - 1].1)
                            .show(ui, |plot_ui| {
                                for (j, i) in self.points.iter().enumerate() {
                                    match j % 2 {
                                        0 => {
                                            if j != 0 {
                                                plot_ui.polygon(
                                                    Polygon::new(
                                                        vec![
                                                            [self.points[j - 1] as f64, 1.0] as _,
                                                            [*i as f64, 1.0] as _,
                                                            [*i as f64, 0.0] as _,
                                                            [self.points[j - 1] as f64, 0.0] as _
                                                        ]
                                                    )
                                                        .fill_alpha(0.5)
                                                        .color(egui::Color32::from_rgb(255, 0, 0))
                                                        .width(0.0)
                                                );
                                            }
                                            plot_ui.vline(
                                                VLine::new(*i)
                                                    .color(egui::Color32::from_rgb(0, 255, 0))
                                                    .name("start jump")
                                            );
                                        }
                                        _ => {
                                            ToF += *i - self.points[j - 1];
                                            if j/2 < 10 && j/2 > 2 {
                                                self.skill_tof[j/2] =  self.points[j - 3]- *i;
                                            }
                                            plot_ui.polygon(
                                                Polygon::new(
                                                    vec![
                                                        [self.points[j - 1] as f64, 2.0] as _,
                                                        [*i as f64, 2.0] as _,
                                                        [*i as f64, 1.0] as _,
                                                        [self.points[j - 1] as f64, 1.0] as _
                                                    ]
                                                )
                                                    .fill_alpha(0.5)
                                                    .color(egui::Color32::from_rgb(0, 255, 0))
                                                    .width(0.0)
                                            );
                                            plot_ui.vline(
                                                VLine::new(*i)
                                                    .color(egui::Color32::from_rgb(255, 0, 0))
                                                    .name("end jump")
                                            );
                                        }
                                    }
                                }

                                plot_ui.vline(
                                    VLine::new((self.drag_one as f32) / framerate)
                                        .highlight(self.current_frame == self.drag_one)
                                        .color(egui::Color32::from_rgb(0, 0, 255))
                                );
                            }).response;

                        ui.horizontal(|ui| {
                            if ui.selectable_label(!self.add_points, "move player").clicked() {
                                self.add_points = false;
                            }
                            if ui.selectable_label(self.add_points, "add points").clicked() {
                                self.add_points = true;
                            }
                        });

                        match bar.hover_pos() {
                            Some(pos) => {
                                ui.separator();
                                self.current_frame = (
                                    (((pos.x - bar.rect.left()) / bar.rect.width()) *
                                        (self.textures.len() as f32)) as usize
                                ).clamp(0, self.textures.len() - 1);
                                if self.add_points {
                                    let mut delete = false;
                                    for pt in 0..self.points.len() {
                                        if self.points.len() <= pt {
                                            break;
                                        }
                                        if
                                            (
                                                self.points[pt] -
                                                timestamp
                                            ).abs() < 0.05
                                        {
                                            delete = true;
                                            ui.label("remove point");
                                            egui_ctx.output_mut(|o| {
                                                o.cursor_icon = egui::CursorIcon::NotAllowed;
                                            });
                                            if ui.input(|i| i.pointer.any_click()) {
                                                self.points.remove(pt);
                                            }
                                        }
                                    }
                                    if !delete {
                                        ui.label("Add point");
                                        if ui.input(|i| i.pointer.any_click()) {
                                            self.points.push(timestamp);
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
                                self.current_frame = self.drag_one.clamp(
                                    0,
                                    self.textures.len() - 1
                                );
                            }
                        }

                        ui.separator();
                        ui.label(
                            format!(
                                "Number of skills: {}",
                                ((self.points.len() as f32) / 2.0).floor()
                            )
                        );
                        ui.horizontal(|ui| {
                            ui.label(format!("ToF: {}", ToF));
                            if
                                ui
                                    .small_button(egui_phosphor::CLIPBOARD_TEXT)
                                    .on_hover_text("Copy to clipboard")
                                    .clicked()
                            {
                                egui_ctx.output_mut(|o| {
                                    o.copied_text = format!("{}", ToF);
                                });
                            }
                        });

                        ui.separator();
                        // number input
                        ui.label(format!("Time: {}", timestamp));
                        ui.checkbox(&mut self.show_video, "Render Video");
                    }
                }

                ui.separator();
                ui.small_button(format!("{} Close Window", egui_phosphor::X))
                    .clicked()
                    .then(|| {
                        self.open = false;
                    });
                ui.small_button(format!("{} Kill Window", egui_phosphor::SKULL))
                    .clicked()
                    .then(|| {
                        self.kill = true;
                    });
            });
    }
    pub fn new() -> Video {
        Video {
            skill_tof: [0.0;10],
            full_size: true,
            add_points: false,
            path: String::from(""),
            open: true,
            textures: Vec::new(),
            current_frame: 0,
            show_video: true,
            kill: false,
            rect: [0.0, 0.0, 0.0, 0.0],
            drag_one: 0,
            timestamps: false,
            points: Vec::new(),
            start_from: 0.0,
            thread: vec![],
            interpolate: false,
            id: UNIX_EPOCH.elapsed().unwrap().as_secs_f32().to_string().replace(".", ""),
        }
    }
}