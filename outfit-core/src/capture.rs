use std::{io::Write, path::Path, f32::consts::PI};
use engage::{
    combat::Kaneko,
    gamedata::{Gamedata, PersonData},
    gamemessage::GameMessage,
    gamevariable::GameVariableManager,
    proc::Bindable,
    unit::Unit,
    unitinfo::{UnitInfo, UnitInfoSide},
    unityengine::{Camera, RenderTexture, Transform, UnityComponent, UnityObject, UnityTransform},
};
use engage::spriteatlasmanager::FaceThumbnail;
use unity::{
    engine::{Color, FilterMode, ImageConversion, Rect, Sprite, SpriteMeshType, Texture2D, Vector2},
    prelude::*,
};
use crate::{clamp_value, UnitAssetMenuData, CAPTURE_DIR, THUMB_DIR};
const PNG: [u8; 8] = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];  // PNG File Sig
const PNG2: [u8; 12] = [0x49, 0x48, 0x44, 0x52, 0, 0, 0, 0xBC, 0, 0, 0, 0x4A];  // IHDR with 188 x 74

#[derive(Default)]
pub struct FacialPositions {
    pub jaw: (f32, f32),
    pub lip: Option<(f32, f32)>,
    pub brows: [Option<(f32, f32)>; 6],
    pub screen_dim: (f32, f32),
}
impl FacialPositions {
    pub fn from_transform(t: &Transform, camera: &Camera, width: f32, height: f32) -> Self {
        let mut face = FacialPositions::default();
        face.screen_dim = (width, height);
        face.jaw = Self::get_position(t, "c_jaw_jnt", camera, width, height).unwrap_or((width, height));
        face.lip = Self::get_position(t, "c_lipLow_jnt", camera, width, height);
        ["l_browIn_jnt", "l_browIn_jnt", "l_browOut_jnt", "r_browIn_jnt", "r_browIn_jnt", "r_browOut_jnt"].iter().enumerate().for_each(|(i, s)|{
            face.brows[i] = Self::get_position(t, s, camera, width, height);
        });
        face
    }
    pub fn get_position(t: &Transform, name: &str, cam: &Camera, x_max: f32, y_max: f32) -> Option<(f32, f32)> {
        Kaneko::find_in_children(t, name.into()).map(|v| {
            let pos = cam.world_to_screen_point(v.get_position());
            (pos.x, pos.y)
        }).filter(|&(x, y)| x < x_max && y < y_max)
    }
    pub fn is_valid(&self) -> bool {
        self.jaw.0 < self.screen_dim.0 && self.jaw.1 < self.screen_dim.1 && self.brows.iter().any(|v| v.is_some())
    }
    pub fn get_xy_position(&self) -> (usize, usize, usize) {
        let y_min = if let Some(lip) = self.lip { (2.0 * self.jaw.1 + lip.1) * 0.33 } else { self.jaw.0 } as usize;
        let y_max = self.brows.iter().flatten().map(|v| v.1 as usize).max().unwrap();
        (self.jaw.0 as usize, y_min, y_max)
    }
}

pub fn capture_unit_info<B: Bindable>(proc: &B, face: bool, assign_face: bool) {
    let camera = UnitInfo::get_face_camera_component(UnitInfoSide::Left);
    let rt = UnitInfo::get_render_texture(UnitInfoSide::Left);
    RenderTexture::set_active(rt);
    let w = rt.get_width() as usize;
    let h = rt.get_height() as usize;
    let texture = Texture2D::instantiate().unwrap();
    texture.ctor(w as i32, h as i32, 4, false);
    let rect = Rect::new(0.0, 0.0, w as f32, h as f32);
    texture.read_pixels_impl_injected(&rect, 0, 0, false);
    texture.apply3();
    let pixels = texture.get_pixels();
    let mut x_pos = vec![];
    let mut x_maxs = vec![];
    for x in 0..h {
        let start = x * w;
        let end = (x + 1) * w;
        if let Some(pos) = pixels[start..end].iter().position(|w| (w.r + w.b + w.g) > 0.1) {
            x_pos.push(pos);
        }
        else { x_pos.push(w); }
        if let Some(pos) = pixels[start..end].iter().rposition(|w| (w.r + w.b + w.g) > 0.1) {
            x_maxs.push(pos);
        }
        else { x_maxs.push(0); }
    }
    let x_min = x_pos.iter().map(|v| *v).min().unwrap_or(0);
    let x_max = x_maxs.iter().map(|v| *v).max().unwrap_or(w);
    let y_min = x_pos.iter().enumerate().find(|(_, x)| **x < w).map(|(i, _)| i).unwrap_or(0);
    let y_max = x_pos.iter().enumerate().rfind(|(i, x)| *i > y_min && *i < h && **x < w).map(|(i, _)| i).unwrap_or(h);
    let y_size = y_max - y_min;
    let texture_cropped = Texture2D::instantiate().unwrap();
    texture_cropped.ctor((x_max - x_min) as i32, y_size as i32, 4, false);
    let mut message = String::new();
    if !face {
        for y in 0..(y_max - y_min) {
            for x in 0..(x_max - x_min) {
                let color = pixels[((y + y_min) * w ) + (x + x_min)].get_gamma();
                texture_cropped.set_pixel(x as i32, y as i32, color);
            }
        }
        if let Some(file) = save_texture_png(texture_cropped, false) {
            message = format!("Screen capture created in '{}'", file);
        }
        else { message ="Unable to capture.".to_string(); }
    }
    else {
        let screen_height = unsafe { get_screen_height(None) } as f32;
        let screen_width = unsafe { get_screen_width(None) } as f32;
        if let Some(char) = UnitInfo::get_instance().map(|v| &v.windows[0].unit_info_window_chara_model.char) {
            if let Some(go) = char.get_game_object() {
                let facial_pos = FacialPositions::from_transform(go.get_transform(), camera, screen_width, screen_height);
                if facial_pos.is_valid() {
                    let (x0, y_min, y_max) = facial_pos.get_xy_position();
                    let y_size = y_max - y_min;
                    let x_size = y_size * 1880 / 740;
                    let x_size2 = x_size / 2;
                    let x_trans_width = y_size / 2;
                    let mut empty_right = 0;
                    let mut empty_left = 0;
                    if x0 > x_size && (x0 + x_size) < w {
                        for dx in 0..y_size {
                            for y in y_min..y_max {
                                if pixels[(y * w) + (x_size2 + x0 - dx)].a < 0.01 { empty_right += 1; }
                                if pixels[(y * w) + (x0 - x_size2 + dx)].a < 0.01 { empty_left += 1; }
                            }
                        }
                        let (x_min, x_max) =
                            if empty_right > empty_left {
                                let shift = 3 * (empty_right - empty_left) / (4 * y_size);
                                (x0 - x_size2 - shift, x0 + x_size2 - shift)
                            } else if empty_right < empty_left {
                                let shift = 3 * (empty_left - empty_right) / (4 * y_size);
                                (x0 - x_size2 + shift, x0 + x_size2 + shift)
                            } else { (x0 - x_size2, x0 + x_size2) };

                        let x_width = x_max - x_min;
                        let trans_factor = 1.0 / (x_trans_width as f32);
                        let x_trans_right = x_size - x_trans_width;
                        let mut raw_face = vec![Color::blank(); x_size * y_size];
                        for y in 0..y_size {
                            for x in 0..x_width {
                                let trans_alpha =
                                    if x < x_trans_width {
                                        let dx = trans_factor * (x as f32);
                                        dx * dx * dx
                                    } else if x > x_trans_right {
                                        let dx = ((x - x_trans_right) as f32) * trans_factor;
                                        let f = 1.0 - dx * dx * 1.5;
                                        clamp_value(f, 0.0, 1.0)
                                    } else { 1.0 };
                                let mut color = pixels[((y + y_min) * w) + (x + x_min)].get_gamma();
                                color.a = color.a * trans_alpha;
                                let idx = x + y * x_size;
                                raw_face[idx] = color;
                            }
                        }
                        let scaled = resize(&raw_face, x_size as i32, y_size as i32, 188, 74);
                        texture_cropped.resize_impl(188, 74);
                        for y in 0..74 { for x in 0..188 { texture_cropped.set_pixel(x as i32, y as i32, scaled[x + y * 188]); } }
                        texture_cropped.apply3();
                        if let Some(file_path) = save_texture_png(texture_cropped, true) {
                            if assign_face {
                                let name = file_path.split("/").last().unwrap();
                                if let Some(unit) = UnitAssetMenuData::get_unit() {
                                    if let Some((active, original, loaded)) = get_unit_face_keys(unit){
                                        let key = format!("G_Face_{}", active);
                                        if !GameVariableManager::exist(key.as_str()) { GameVariableManager::make_entry_str(key.as_str(), name); }
                                        else { GameVariableManager::set_string(key.as_str(), name); }
                                        let rect = Rect::new(0.0, 0.0, 188.0, 74.0);
                                        let pivot = Vector2::new(0.5, 0.5);
                                        texture_cropped.set_filter_mode(FilterMode::Trilinear);
                                        let sprite = Sprite::create2(texture_cropped, rect, pivot, 100.0, 1, SpriteMeshType::Tight);
                                        // Check if load key is not the original sprite and then destroy it
                                        if let Some(original) = FaceThumbnail::get_item(&original){
                                            if let Some(alt) =  FaceThumbnail::get_item(&loaded) {
                                                if !original.equal(alt) && !original.equals(alt) { alt.destroy(); }
                                            }
                                        }
                                        FaceThumbnail::try_insert(&active, sprite);
                                        FaceThumbnail::try_insert(&loaded, sprite);
                                        GameMessage::create_key_wait(proc, format!("Assigned and saved face thumbnail to\n'{}'.", file_path.as_str()));
                                        return;
                                    }
                                }
                            }
                            message = format!("Save face thumbnail to {}.", file_path.as_str());
                        }
                        else { message = "Unable to save face thumbnail to file.".to_string(); }
                    }
                }
            }
        }
        else { message = "Unable to save face thumbnail to file.".to_string(); }
    }
    GameMessage::create_key_wait(proc, message);
    texture_cropped.destroy();
}
/// Taken from https://docs.rs/image/latest/src/image/imageops/sample.rs.html
fn resize(data: &Vec<Color>, old_w: i32, old_h: i32, new_w: i32, new_h: i32) -> Vec<Color> {
    let filter_value = 4.0;
    let width = old_w as usize;
    let height = old_h as usize;
    let new_width = new_w as usize;
    let new_height = new_h as usize;
    let mut ws = vec![];
    let v_sample =
        if new_height == height { data.clone() }
        else {
            let ratio = old_h as f32 / new_h as f32;
            let s_ratio = if ratio < 1.0 { 1.0 } else { ratio };
            let src_support = filter_value  * s_ratio;
            let mut v_sample: Vec<Color> = vec![Color::blank(); width * new_height];
            for out_y in 0..new_height {
                let input_y = (out_y as f32 + 0.5) * ratio;
                let left = (input_y - src_support).floor() as i32;
                let left = clamp_value(left, 0, old_h - 1 ) as usize;
                let right = (input_y + src_support).ceil() as i32;
                let right = clamp_value(right, (left as i32) + 1, old_h) as usize;
                let input_y = input_y - 0.5;
                let mut sum = 0.0;
                ws.clear();
                for i in left..right {
                    let w = lanczos((i as f32 - input_y) / s_ratio, 6.0);
                    ws.push(w);
                    sum += w;
                }
                ws.iter_mut().for_each(|w| *w /= sum);
                for x in 0..width {
                    let mut c = Color::blank();
                    ws.iter().enumerate().for_each(|(i, w)| {
                        let c_idx = x + (left + i) * width;
                        let color = data[c_idx];
                        c.r += w * color.r;
                        c.g += w * color.g;
                        c.b += w * color.b;
                        c.a += w * color.a;
                    });
                    v_sample[x + out_y * width] = c;
                }
            }
            v_sample
        };
    let ratio = (old_w as f32) / (new_w as f32);
    let s_ratio = if ratio < 1.0 { 1.0 } else { ratio };
    let src_support = filter_value  * s_ratio;
    let mut out: Vec<Color> = vec![Color::blank(); new_height * new_width];
    for out_x in 0..new_width {
        let input_x = (out_x as f32 + 0.5) * ratio;
        let left = (input_x - src_support).floor() as i32;
        let left = clamp_value(left, 0, old_w - 1 ) as usize;
        let right = (input_x + src_support).ceil() as i32;
        let right = clamp_value(right, left as i32 + 1, old_w ) as usize;
        let input_x = input_x - 0.5;
        let mut sum = 0.0;
        ws.clear();
        for i in left..right {
            let w = lanczos((i as f32 - input_x) / s_ratio, 6.0);
            ws.push(w);
            sum += w;
        }
        ws.iter_mut().for_each(|w| *w /= sum);
        for y in 0..new_height {
            let mut c = Color::blank();
            ws.iter().enumerate().for_each(|(i, w)| {
                let c_idx = (left + i) + y * width;
                let color = v_sample[c_idx];
                c.r += w * color.r;
                c.g += w * color.g;
                c.b += w * color.b;
                c.a += w * color.a;
            });
            out[out_x + (y * new_width)] = c;
        }
    }
    out
}
pub fn save_texture_png(texture2d: &Texture2D, is_face: bool) -> Option<String> {
    let data = texture2d.encode_to_png();
    if let Some(unit) = UnitAssetMenuData::get_unit() {
        let name = unit.get_name();
        let path = if is_face { THUMB_DIR } else { CAPTURE_DIR };
        let file_path = crate::get_next_filename(path, &name.to_string(), "png");
        if let Ok(mut file) = std::fs::File::options().create(true).write(true).truncate(true).open(file_path.as_str()){
            let result = file.write_all(&data);
            if result.is_ok(){ return Some(file_path); }
        }
    }
    None
}
/// Gets Dictionary Keys from Unit's Ascii name
/// - active (ascii name) used in game
/// - original (o_ + ascii name) the original sprite
/// - loaded (a_ + ascii_name) the replacement sprite
pub fn get_unit_face_keys(unit: &Unit) -> Option<(String, String, String)> {
    if let Some(ascii_name) = unit.person.get_ascii_name() {
        let mut active = ascii_name.to_string();
        let mut original = format!("o_{}", ascii_name);
        let mut loaded = format!("a_{}", ascii_name);

        if unit.person.parent.index == 1 && unit.edit.gender == 2 {
            active.push('W');
            loaded.push('W');
            original.push('W');
        }
        Some((active, original, loaded))
    }
    else { None }
}
pub fn update_face(unit: &Unit, use_original: bool){
    if let Some((active, loaded, original)) = get_unit_face_keys(unit) {
        if let Some(sprite) = if use_original { FaceThumbnail::get_item(original) } else { FaceThumbnail::get_item(loaded)}{
            FaceThumbnail::try_insert(active, sprite);
        }
    }
}
pub fn reset_faces(title: bool) {
    let s = FaceThumbnail::get_static_fields().face_thumb.cache_table.entries.iter()
        .filter(|i| i.key.is_some_and(|a| a.to_string().starts_with("o_")))
        .map(|c| c.key.unwrap().to_string())
        .collect::<Vec<String>>();

    s.iter().for_each(|o|{
        if let Some(original_sprite) = FaceThumbnail::get_item(o){
            let active = o.trim_start_matches("o_").to_string();
            let load = format!("a_{}", active);
            if let Some(loaded) = FaceThumbnail::get_item(&load) {
                if !loaded.equals(original_sprite) && !loaded.equal(original_sprite) {
                    FaceThumbnail::try_insert(&load, original_sprite);
                    FaceThumbnail::try_insert(&active, original_sprite);
                    loaded.destroy();
                }
            }
        }
    });
    if !UnitAssetMenuData::get().is_loaded && !title {
        UnitAssetMenuData::get().data.iter().for_each(|d|{
            if let Some(person_data) = PersonData::try_get_hash(d.person) {
                if let Some(v) = person_data.get_ascii_name() {
                    let ascii = if d.flag & 16 == 0 { v.to_string() } else { format!("{}W", v) };
                    load_png_to_by_ascii(&ascii, d.flag & 8 != 0 );
                }
            }
        });
    }
}
fn load_png_to_by_ascii(ascii: &String, use_sprite: bool) -> bool {
    let file_key = format!("G_Face_{}", ascii);
    if GameVariableManager::exist(&file_key) {
        let file = GameVariableManager::get_string(&file_key).to_string();
        if !file.starts_with("---") && file.contains(".png") {
            let p = format!("{}{}", THUMB_DIR, GameVariableManager::get_string(&file_key));
            let path = Path::new(p.as_str());
            if path.exists() {
                if let Some(mut file) = std::fs::read(path).ok().filter(|d| png_file_check(d)){
                    if let Some(sprite) = create_face_sprite(&mut file) {
                        let alt_key = format!("a_{}", ascii);
                        if FaceThumbnail::try_insert(alt_key, sprite) {
                            if use_sprite { FaceThumbnail::try_insert(ascii.as_str(), sprite); }
                        }
                        return true;
                    }
                }
            }
            else { GameVariableManager::set_string(&file_key, "---"); }
        }
    }
    false
}
pub fn create_face_sprite(data: &mut Vec<u8>) -> Option<&'static Sprite> {
    let data = Il2CppArray::from_slice(data).ok()?;
    let new_texture = Texture2D::new(188, 74);
    if ImageConversion::load_image(new_texture, data) {
        new_texture.set_filter_mode(FilterMode::Trilinear);
        let rect = Rect::new(0.0, 0.0, 188.0, 74.0);
        let pivot = Vector2::new(0.5, 0.5);
        let sprite = Sprite::create2(new_texture, rect, pivot, 100.0, 1, SpriteMeshType::Tight);
        Some(sprite)
    }
    else { None }
}
pub(crate) fn png_file_check(file: &Vec<u8>) -> bool {
    if file.len() < 24 { return false; }
    for x in 0..8 { if file[x] != PNG[x] { return false; } }
    for x in 0..12 { if file[x+12] != PNG2[x] { return false; } }
    true
}
fn sinc(t: f32) -> f32 {
    let a = t * PI;
    if t == 0.0 { 1.0 } else { a.sin() / a }
}
fn lanczos(x: f32, t: f32) -> f32 {
    if x.abs() < t { sinc(x) * sinc(x / t) } else { 0.0 }
}
#[skyline::from_offset(0x2f8b960)]
fn get_screen_height(optional_method: OptionalMethod) -> i32;

#[skyline::from_offset(0x2f8b920)]
fn get_screen_width(optional_method: OptionalMethod) -> i32;