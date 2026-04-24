use std::f32::consts::PI;
use std::io::Write;
use std::path::Path;
use engage::combat::Kaneko;
use engage::gamedata::{Gamedata, PersonData};
use engage::gamemessage::GameMessage;
use engage::gamevariable::GameVariableManager;
use engage::proc::Bindable;
use engage::spriteatlasmanager::FaceThumbnailStaticFields;
use engage::unit::Unit;
use engage::unitinfo::{UnitInfo, UnitInfoSide};
use engage::unityengine::{RenderTexture, UnityComponent, UnityObject, UnityTransform};
use unity::engine::{Color, FilterMode, ImageConversion, Rect, Sprite, SpriteMeshType, Texture2D, Vector2};
use unity::macro_context::Il2CppClassData;
use unity::prelude::{ArrayInstantiator, Il2CppArray, OptionalMethod};
use unity::system::{Dictionary, Il2CppString, InsertionBehavior};
use crate::{clamp_value, UnitAssetMenuData, CAPTURE_DIR, THUMB_DIR};

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
        } else {
            x_pos.push(w);
        }
        if let Some(pos) = pixels[start..end].iter().rposition(|w| (w.r + w.b + w.g) > 0.1) { x_maxs.push(pos); } else { x_maxs.push(0); }
    }
    let x_min = x_pos.iter().map(|v| *v).min().unwrap_or(0);
    let x_max = x_maxs.iter().map(|v| *v).max().unwrap_or(w);
    let y_min = x_pos.iter().enumerate().find(|(_, x)| **x < w).map(|(i, _)| i).unwrap_or(0);
    let y_max = x_pos.iter().enumerate().rfind(|(i, x)| *i > y_min && *i < h && **x < w).map(|(i, _)| i).unwrap_or(h);
    let y_size = y_max - y_min;
    let texture_cropped = Texture2D::instantiate().unwrap();
    texture_cropped.ctor((x_max - x_min) as i32, y_size as i32, 4, false);
    if !face {
        for y in 0..(y_max - y_min) {
            for x in 0..(x_max - x_min) {
                let color = pixels[((y + y_min) * w ) + (x + x_min)].get_gamma();
                texture_cropped.set_pixel(x as i32, y as i32, color);
            }
        }
        if let Some(file) = save_texture_png(texture_cropped, false) {
            GameMessage::create_key_wait(proc, format!("Screen capture created in '{}'", file))
        }
        else {
            GameMessage::create_key_wait(proc, "Unable to capture.");
        }
        texture_cropped.destroy();
        return;
    }
    else {
        let screen_height = unsafe { get_screen_height(None) } as f32;
        let screen_width = unsafe { get_screen_width(None) } as f32;
        if let Some(char) = UnitInfo::get_instance().map(|v| &v.windows[0].unit_info_window_chara_model.char) {
            if let Some(go) = char.get_game_object() {
                let jaw_pos = Kaneko::find_in_children(go.get_transform(), "c_jaw_jnt".into()).map(|v| {
                    let pos = camera.world_to_screen_point(v.get_position());
                    (pos.x, pos.y)
                });
                let y_max =
                    ["l_browIn_jnt", "l_browIn_jnt", "l_browOut_jnt", "r_browIn_jnt", "r_browIn_jnt", "r_browOut_jnt"].iter().flat_map(|v|
                        Kaneko::find_in_children(go.get_transform(), v.into()).map(|v| camera.world_to_screen_point(v.get_position()).y as usize)
                    ).max().filter(|x| *x < screen_height as usize);
                if let Some((jaw_pos, y_max)) = jaw_pos.filter(|v| v.1 > 0.0 && v.1 < screen_height && v.0 > 0.0 && v.0 < screen_width).zip(y_max) {
                    let x0 = jaw_pos.0 as usize;
                    let y_min = jaw_pos.1 as usize;
                    let y_size = y_max - y_min;
                    let x_size = y_size * 1880 / 740;
                    let x_size2 = x_size / 2;
                    let x_trans_width = y_size / 3;
                    // adjusting center by calculating average number of blank columns
                    let mut empty_right = 0;
                    let mut empty_left = 0;
                    if x0 > x_size && (x0 + x_size) < w {
                        for dx in 0..y_size {
                            for y in y_min..y_max {
                                if pixels[(y * w) + (x_size2 + x0 - dx)].a < 0.01 { empty_right += 1; }
                                if pixels[(y * w) + (x0 + dx)].a < 0.01 { empty_left += 1; }
                            }
                        }
                        let (x_min, x_max) =
                        if empty_right > empty_left {
                            let shift = (empty_right - empty_left) / y_size;
                            (x0 - x_size2 - shift, x0 + x_size2 - shift)
                        }
                        else if empty_right < empty_left {
                            let shift = (empty_left - empty_right) / y_size;
                            (x0 - x_size2 + shift, x0 + x_size2 + shift)
                        }
                        else { (x0 - x_size2, x0 + x_size2) };

                        let x_width = x_max - x_min;
                        let trans_factor = 1.0 / (x_trans_width as f32);
                        let x_trans_right = x_size - x_trans_width;
                        let mut raw_face = vec![Color::blank(); x_size * y_size];
                        for y in 0..y_size {
                            for x in 0..x_width {
                                // adding transparency on left and right sides.
                                let trans_alpha =
                                    if x < x_trans_width {
                                        let dx = trans_factor * (x as f32 );
                                        dx*dx*dx
                                    }
                                    else if x > x_trans_right {
                                        let dx = ((x - x_trans_right) as f32 ) * trans_factor;
                                        1.0 - dx
                                    }
                                    else { 1.0 };
                                /*
                                    if x < x_trans_width { x as f32 * trans_factor }
                                    else if x_trans_right > x { 1.0 - (x - x_trans_right) as f32 * trans_factor }
                                    else { 1.0 };

                                 */
                                let mut color = pixels[((y + y_min) * w) + (x + x_min)].get_gamma();
                                color.a = color.a * trans_alpha;
                                let idx = x + y * x_size;
                                raw_face[idx] = color;
                            }
                        }
                        let scaled = resize(&raw_face, x_size as i32, y_size as i32, 188, 74);
                        texture_cropped.resize_impl(188, 74);
                        for y in 0..74 {
                            for x in 0..188 { texture_cropped.set_pixel(x as i32, y as i32, scaled[x + y * 188]); }
                        }
                        texture_cropped.apply3();
                        if let Some(file_path) = save_texture_png(texture_cropped, true) {
                            if assign_face {
                                let name = file_path.split("/").last().unwrap();
                                if let Some(unit) = UnitAssetMenuData::get_unit() {
                                    if let Some(ascii_name) = unit.person.get_ascii_name() {
                                        let table_key;
                                        let original_key;
                                        if unit.person.parent.index == 1 && unit.edit.gender == 2 {
                                            table_key = format!("a_{}W", ascii_name);
                                            original_key = format!("{}W", ascii_name);
                                        }
                                        else {
                                            table_key = format!("a_{}", ascii_name);
                                            original_key = ascii_name.to_string();
                                        }
                                        let key = format!("G_Face_{}", original_key);
                                        println!("Key: {} => {}", key, name);
                                        if !GameVariableManager::exist(key.as_str()) { GameVariableManager::make_entry_str(key.as_str(), name); }
                                        else { GameVariableManager::set_string(key.as_str(), name); }
                                        let rect = Rect::new(0.0, 0.0, 188.0, 74.0);
                                        let pivot = Vector2::new(0.5, 0.5);
                                        texture_cropped.set_filter_mode(FilterMode::Trilinear);
                                        let sprite = Sprite::create2(texture_cropped, rect, pivot, 100.0, 1, SpriteMeshType::Tight);
                                        let face_thumbs = &engage::spriteatlasmanager::FaceThumbnail::class().get_static_fields_mut::<FaceThumbnailStaticFields>().face_thumb;
                                        if let Some(original) = face_thumbs.cache_table.get_item(format!("o_{}", original_key).into()){
                                            if let Some(alt) = face_thumbs.cache_table.get_item(table_key.as_str().into()){
                                                if !original.equal(alt) && !original.equals(alt) {
                                                    alt.destroy();
                                                    println!("Destroy Alt");
                                                }
                                                face_thumbs.cache_table.try_insert(table_key.as_str().into(), sprite, InsertionBehavior::Overwrite);
                                                face_thumbs.cache_table.try_insert(original_key.as_str().into(), sprite, InsertionBehavior::Overwrite);
                                                GameMessage::create_key_wait(proc, format!("Assigned and saved face thumbnail to\n'{}'.", file_path.as_str()));
                                            }
                                        }
                                        return;
                                    }
                                }
                            }
                            else { texture_cropped.destroy(); }
                            GameMessage::create_key_wait(proc, format!("Save face thumbnail to {}.", file_path.as_str()));
                        }
                        else { GameMessage::create_key_wait(proc, "Unable to save thumbnail."); }
                    }
                }
                else {
                    GameMessage::create_key_wait(proc, "Unable to find face for thumbnail.");
                }
            }
        }
    }
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
        let mut file_path = format!("{}{}.png", path, name);
        if Path::new(file_path.as_str()).exists() {
            let mut c = 1;
            loop {
                file_path = format!("{}{}-{}.png", path, name, c);
                if Path::new(file_path.as_str()).exists() { c += 1; } else { break; }
            }
        }
        if let Ok(mut file) = std::fs::File::options().create(true).write(true).truncate(true).open(file_path.as_str()){
            let result = file.write_all(&data);
            if result.is_ok(){ return Some(file_path); }
        }
    }
    None
}
pub fn get_unit_face_keys(unit: &Unit) -> Option<(String, String, String)> {
    if let Some(ascii_name) = unit.person.get_ascii_name() {
        let mut active = ascii_name.to_string();
        let mut loaded = format!("a_{}", ascii_name);
        let mut original = format!("o_{}", ascii_name);
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
    let mut r = false;
    if let Some((active, loaded, original)) = get_unit_face_keys(unit) {
        let face_thumbs = &engage::spriteatlasmanager::FaceThumbnail::class().get_static_fields_mut::<FaceThumbnailStaticFields>().face_thumb;
        let sprite = if use_original { face_thumbs.cache_table.get_item(original.as_str().into()) } else { face_thumbs.cache_table.get_item(loaded.as_str().into()) };
        if let Some(sprite) = sprite {
            println!("Found Sprite");
            r = face_thumbs.cache_table.try_insert(active.as_str().into(), sprite, InsertionBehavior::Overwrite);
        }
    }
    println!("Updated {}'s face: {}", unit.get_name(), r);
}
pub fn reset_faces(title: bool) {
    println!("Resetting Faces");
    let thumbs = &engage::spriteatlasmanager::FaceThumbnail::class().get_static_fields_mut::<FaceThumbnailStaticFields>().face_thumb;
    let s = thumbs.cache_table.entries.iter().filter(|i| i.key.is_some_and(|a| a.to_string().starts_with("o_"))).map(|c| c.key.unwrap().to_string()).collect::<Vec<String>>();
    s.iter().for_each(|o|{
        if let Some(original_sprite) = thumbs.cache_table.get_item(o.as_str().into()) {
            println!("Found Original: {}", o);
            let active_key = o.trim_start_matches("o_").to_string();
            let alt_key = format!("a_{}", active_key);
            if let Some(alt) = thumbs.cache_table.get_item(alt_key.as_str().into()) {
                println!("Active / Alt: {} / {}", active_key, alt_key);
                if !alt.equals(original_sprite) && !alt.equal(original_sprite) {
                    thumbs.cache_table.try_insert(alt_key.as_str().into(), original_sprite, InsertionBehavior::Overwrite);
                    thumbs.cache_table.try_insert(active_key.as_str().into(), original_sprite, InsertionBehavior::Overwrite);
                    alt.destroy();
                    println!("Destroyed: {}", alt_key);
                }
            }
        }
    });
    if !UnitAssetMenuData::get().is_loaded && !title {
        UnitAssetMenuData::get().data.iter().for_each(|d|{
            if let Some(person_data) = PersonData::try_get_hash(d.person) {
                if let Some(v) = person_data.get_ascii_name() {
                    let ascii = if d.flag & 16 == 0 { v.to_string() } else { format!("{}W", v) };
                    load_png_to_by_ascii(&ascii, thumbs.cache_table, d.flag & 8 != 0 );
                }
            }
        });
    }
}
fn load_png_to_by_ascii(ascii: &String, table: &Dictionary<'static, &'static Il2CppString, &'static Sprite>, use_sprite: bool) -> bool {
    let file_key = format!("G_Face_{}", ascii);
    if GameVariableManager::exist(&file_key) {
        let file = GameVariableManager::get_string(&file_key).to_string();
        if !file.starts_with("---") && file.contains(".png") {
            let p = format!("{}{}", THUMB_DIR, GameVariableManager::get_string(&file_key));
            let path = Path::new(p.as_str());
            if path.exists() {
                if let Ok(file) = std::fs::read(path) {
                    println!("Loaded: {}", p);
                    let data = Il2CppArray::from_slice(file).unwrap();
                    let new_texture = Texture2D::new(188, 74);
                    if ImageConversion::load_image(new_texture, data) {
                        new_texture.set_filter_mode(FilterMode::Trilinear);
                        let rect = Rect::new(0.0, 0.0, 188.0, 74.0);
                        let pivot = Vector2::new(0.5, 0.5);
                        let sprite = Sprite::create2(new_texture, rect, pivot, 100.0, 1, SpriteMeshType::Tight);
                        let alt_key = format!("a_{}", ascii);
                        if table.try_insert(alt_key.as_str().into(), sprite, InsertionBehavior::Overwrite) {
                            if use_sprite { table.try_insert(ascii.as_str().into(), sprite, InsertionBehavior::Overwrite); }
                            println!("Loaded Face Sprite in cache");
                            return true;
                        }
                    }
                }
            }
            else { GameVariableManager::set_string(&file_key, "---"); }
        }
    }
    false
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