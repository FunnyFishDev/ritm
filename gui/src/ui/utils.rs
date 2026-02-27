use crate::ui::constant::Constant;
use eframe::egui::{Pos2, Vec2};

/// Compute the distance between 2 points
pub fn distance(p1: Pos2, p2: Pos2) -> f32 {
    f32::sqrt((p2.x - p1.x).powi(2) + (p2.y - p1.y).powi(2))
}

/// Compute the repulsion force of the node
pub fn rep_force(p1: Pos2, p2: Pos2) -> f32 {
    let force = Constant::CREP / distance(p1, p2).powi(2);
    if force > Constant::MAX_FORCE {
        Constant::MAX_FORCE * force.signum()
    } else {
        force
    }
}

/// Compute the attraction force of the node
pub fn attract_force(p1: Pos2, p2: Pos2, size: f32) -> f32 {
    let force = Constant::CSPRING * (distance(p1, p2) / size).log(10.0);
    if force > Constant::MAX_FORCE {
        Constant::MAX_FORCE * force.signum()
    } else {
        force
    }
}

/// Compute the direction between 2 points
pub fn direction(p1: Pos2, p2: Pos2) -> Vec2 {
    Vec2::new(p2.x - p1.x, p2.y - p1.y).normalized()
}

pub type FileData = Vec<u8>;

// wasm
#[cfg(target_arch = "wasm32")]
use js_sys::{Array, ArrayBuffer, Uint8Array};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use web_sys::{File, FileReader, HtmlInputElement, Url, window};

#[cfg(target_arch = "wasm32")]
pub struct FileDialog {
    tx: std::sync::mpsc::Sender<FileData>,
    rx: std::sync::mpsc::Receiver<FileData>,
    input: HtmlInputElement,
    closure: Option<Closure<dyn FnMut()>>,
}

#[cfg(target_arch = "wasm32")]
impl Default for FileDialog {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();

        let document = window().unwrap().document().unwrap();
        let body = document.body().unwrap();
        let input = document
            .create_element("input")
            .unwrap()
            .dyn_into::<HtmlInputElement>()
            .unwrap();
        input.set_attribute("type", "file").unwrap();
        input.style().set_property("display", "none").unwrap();
        body.append_child(&input).unwrap();

        Self {
            rx,
            tx,
            input,
            closure: None,
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl Drop for FileDialog {
    fn drop(&mut self) {
        self.input.remove();
        if self.closure.is_some() {
            std::mem::replace(&mut self.closure, None).unwrap().forget();
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl FileDialog {
    pub fn open(&mut self) {
        if let Some(closure) = &self.closure {
            self.input
                .remove_event_listener_with_callback("change", closure.as_ref().unchecked_ref())
                .unwrap();
            std::mem::replace(&mut self.closure, None).unwrap().forget();
        }

        let tx = self.tx.clone();
        let input_clone = self.input.clone();

        let closure = Closure::once(move || {
            if let Some(file) = input_clone.files().and_then(|files| files.get(0)) {
                let reader = FileReader::new().unwrap();
                let reader_clone = reader.clone();
                let onload_closure = Closure::once(Box::new(move || {
                    let array_buffer = reader_clone
                        .result()
                        .unwrap()
                        .dyn_into::<ArrayBuffer>()
                        .unwrap();
                    let buffer = Uint8Array::new(&array_buffer).to_vec();
                    tx.send(buffer).ok();
                }));

                reader.set_onload(Some(onload_closure.as_ref().unchecked_ref()));
                reader.read_as_array_buffer(&file).unwrap();
                onload_closure.forget();
            }
        });

        self.input
            .add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())
            .unwrap();
        self.closure = Some(closure);
        self.input.click();
    }

    pub fn get(&self) -> Option<Vec<u8>> {
        if let Ok(file) = self.rx.try_recv() {
            Some(file)
        } else {
            None
        }
    }

    pub fn save(&self, filename: &str, filedata: FileData) {
        let array = Uint8Array::from(filedata.as_slice());
        let blob_parts = Array::new();
        blob_parts.push(&array.buffer());

        let file = File::new_with_blob_sequence_and_options(
            &blob_parts.into(),
            filename,
            web_sys::FilePropertyBag::new().type_("application/octet-stream"),
        )
        .unwrap();
        let url = Url::create_object_url_with_blob(&file);
        if let Some(window) = web_sys::window() {
            window.location().set_href(&url.unwrap()).ok();
        }
    }
}

// native

#[cfg(not(target_arch = "wasm32"))]
use rfd;

#[cfg(not(target_arch = "wasm32"))]
#[derive(Default)]
pub struct FileDialog {
    file: Option<FileData>,
}

#[cfg(not(target_arch = "wasm32"))]
impl FileDialog {
    pub fn open(&mut self) {
        let path = rfd::FileDialog::new().pick_file();
        if let Some(path) = path {
            self.file = std::fs::read(path).ok();
        }
    }

    pub fn get(&mut self) -> Option<FileData> {
        self.file.take()
    }

    pub fn save(&self, filename: &str, file: FileData) {
        let path = rfd::FileDialog::new().set_file_name(filename).save_file();

        if let Some(path) = path {
            std::fs::write(path, file).ok();
        }
    }
}

pub(crate) mod fade {
    use std::collections::BinaryHeap;

    use egui::{Color32, Direction, Mesh, Rect, Ui, pos2};

    #[derive(Debug, Clone)]
    pub struct ColorKey {
        color: Color32,
        key: f32,
    }

    impl Eq for ColorKey {}

    impl PartialEq for ColorKey {
        fn eq(&self, other: &Self) -> bool {
            self.key == other.key
        }
    }

    impl PartialOrd for ColorKey {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for ColorKey {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            other.key.total_cmp(&self.key)
        }
    }

    pub struct Fade {
        color: BinaryHeap<ColorKey>,
        step: usize,
        min: f32,
        max: f32,
    }

    impl Fade {
        pub fn new() -> Self {
            Self {
                color: BinaryHeap::new(),
                step: 100,
                min: f32::INFINITY,
                max: f32::NEG_INFINITY,
            }
        }

        pub fn with_color(mut self, color: Color32, key: f32) -> Self {
            self.color.push(ColorKey { color, key });
            if key < self.min {
                self.min = key
            }
            if key > self.max {
                self.max = key
            }
            self
        }

        pub fn with_step(mut self, step: usize) -> Self {
            self.step = step;
            self
        }
    }

    pub fn fade(ui: &mut Ui, rect: Rect, dir: Direction, fade: Fade) {
        let mut mesh = Mesh::default();

        let step_size = match dir {
            Direction::LeftToRight | Direction::RightToLeft => rect.width() / fade.step as f32,
            Direction::TopDown | Direction::BottomUp => rect.height() / fade.step as f32,
        };
        let mut old_max = match dir {
            Direction::LeftToRight => rect.min.x,
            Direction::RightToLeft => rect.max.x,
            Direction::TopDown => rect.min.y,
            Direction::BottomUp => rect.max.y,
        };

        let sorted = fade.color.clone().into_sorted_vec();
        let mut colors = sorted
            .iter()
            .zip(sorted.iter().skip(1))
            .map(|(c1, c2)| (c2, c1))
            .collect::<Vec<(&ColorKey, &ColorKey)>>();
        let mut color = colors.pop().expect("exist");

        let mut i = 0;
        while i < fade.step {
            let r = match dir {
                Direction::LeftToRight => {
                    let r = Rect::from_min_max(
                        pos2(old_max, rect.min.y),
                        pos2(old_max + step_size, rect.max.y),
                    );
                    old_max += step_size;
                    r
                }
                Direction::RightToLeft => {
                    let r = Rect::from_min_max(
                        pos2(old_max - step_size, rect.min.y),
                        pos2(old_max, rect.max.y),
                    );
                    old_max -= step_size;
                    r
                }
                Direction::TopDown => {
                    let r = Rect::from_min_max(
                        pos2(rect.min.x, old_max),
                        pos2(rect.max.x, old_max + step_size),
                    );
                    old_max += step_size;
                    r
                }
                Direction::BottomUp => {
                    let r = Rect::from_min_max(
                        pos2(rect.min.x, old_max - step_size),
                        pos2(rect.max.x, old_max),
                    );
                    old_max -= step_size;
                    r
                }
            };
            let x = fade.min + ((fade.max - fade.min) * i as f32 / fade.step as f32);
            let t = (x - color.0.key) / (color.1.key - color.0.key);
            mesh.add_colored_rect(r, color.0.color.lerp_to_gamma(color.1.color, t));

            if x >= color.1.key {
                println!("skipping !");
                color = colors.pop().expect("lol");
            }
            i += 1;
        }
        ui.painter().add(mesh);
    }
}
