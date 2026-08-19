use std::{io::Write, path::Path, f32::consts::PI};
use engage::{
    unity_engine::{
        IGameObjectMethods, ICameraMethods, IComponentMethods,
        IRenderTextureMethods, ITexture2DMethods, ITextureMethods,
        ITransformMethods, TextureFormat, Sprite, Color
    },
    app::{
        unit::*, unitinfo::*, gameuserdata::*, gamevariable::*,
        ISingletonClass_1Methods, IPersonDataMethods,
        ISingletonProcInst_1Methods, IStructBase,
        IUnitEdit, IUnitInfoWindowCharaModel,
        ISpriteAtlasManager_2, IStructData_1Methods
    },
    system::{
        collections::generic::{IDictionary_2Methods, InsertionBehavior}, IObjectMethods
    },
    Dictionary_2Ext,
};
use unity::Cast;
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
    pub fn from_transform(t: engage::unity_engine::Transform, camera: engage::unity_engine::Camera, width: f32, height: f32) -> Self {
        let mut face = FacialPositions::default();
        face.screen_dim = (width, height);
        face.jaw = Self::get_position(t, "c_jaw_jnt", camera, width, height).unwrap_or((width, height));
        face.lip = Self::get_position(t, "c_lipLow_jnt", camera, width, height);
        ["l_browIn_jnt", "l_browIn_jnt", "l_browOut_jnt", "r_browIn_jnt", "r_browIn_jnt", "r_browOut_jnt"].iter().enumerate().for_each(|(i, s)|{
            face.brows[i] = Self::get_position(t, s, camera, width, height);
        });
        face
    }
    pub fn get_position(t: engage::unity_engine::Transform, name: &str, cam: engage::unity_engine::Camera, _x_max: f32, _y_max: f32) -> Option<(f32, f32)> {
        let t2 = engage::combat::Kaneko::find_in_children(t, name);
        if !t2.is_null() {
            let pos = cam.world_to_screen_point_2(t2.get_position());
            Some((pos.x, pos.y))
        }
        else { None }
    }
    pub fn is_valid(&self) -> bool { self.jaw.0 < self.screen_dim.0 && self.jaw.1 < self.screen_dim.1 && self.brows.iter().any(|v| v.is_some()) }
    pub fn get_xy_position(&self) -> (usize, usize, usize) {
        let y_min = if let Some(lip) = self.lip { (2.0 * self.jaw.1 + lip.1) * 0.33 } else { self.jaw.0 } as usize;
        let y_max = self.brows.iter().flatten().map(|v| v.1 as usize).max().unwrap();
        (self.jaw.0 as usize, y_min, y_max)
    }
}

pub fn capture_unit_info(proc: impl Into<engage::app::ProcInst>, face: bool, assign_face: bool) {
    let camera = UnitInfo::get_face_camera_component(UnitInfo_Side::left());
    let rt = UnitInfo::get_render_texture(UnitInfo_Side::left());
    engage::unity_engine::RenderTexture::set_active(rt);
    let w = IRenderTextureMethods::get_width(rt) as usize;
    let h = IRenderTextureMethods::get_height(rt) as usize;
    let texture = engage::unity_engine::Texture2D::new_8(w as i32, h as i32, TextureFormat{value: 4}, false);
    let rect = engage::unity_engine::Rect{
        m_x_min: 0.0,
        m_y_min: 0.0,
        m_width: w as f32,
        m_height: h as f32,
    };
    texture.read_pixels_impl(rect, 0, 0, false);
    texture.apply_3();
    let pixels = texture.get_pixels_4();
    let mut x_min = w;
    let mut x_max = 0;
    let mut y_min = 0;
    let mut y_max = 0;
    for x in 0..h {
        let start = x * w;
        let mut pc = 0;
        for ww in 0..w { // Left Bound
            let p1 = pixels.get(start + ww);
            if p1.r > 0.0 || p1.b > 0.0 || p1.g > 0.0 {
                pc += 1;
                if ww > x_max { x_max = ww; }
            }
        }
        if pc > 10 && y_min == 0 { y_min = x; }
        if pc == 0 && y_min > 0 && y_max == 0 { y_max = x; }
    }
    if y_min == 0 || x_min == 0 {
        engage::app::GameMessage::create_key_wait(proc, "Capture is empty.");
        return;
    }
    for x in 0..h {
        let end = (x + 1) * w;
        for ww in (w - x_min)..w { // Left Bound
            let x_right = w - ww;
            let p1 = pixels.get(end - ww);
            if p1.r > 0.0 || p1.b > 0.0 || p1.g > 0.0 { if x_right < x_min  { x_min = x_right; } }
        }
    }
    let y_size = y_max - y_min;
    let texture_cropped = engage::unity_engine::Texture2D::new_8((x_max - x_min) as i32, y_size as i32,TextureFormat{value: 4}, false);
    let mut message = String::new();
    if !face {
        for y in 0..(y_max - y_min) {
            for x in 0..x_max - x_min{
                let index = ((y + y_min) * w ) + (x + x_min);
                let mut color = pixels.get(index);
                texture_cropped.set_pixel(x as i32, y as i32,color.get_gamma());
            }
        }
        if let Some(file) = save_texture_png(texture_cropped, false) {
            message = format!("Screen capture created in '{}'", file);
        }
        else { message ="Unable to save capture.\nMissing directory?".to_string(); }
    }
    else {
        let screen_height = engage::unity_engine::Screen::get_height() as f32;
        let screen_width =  engage::unity_engine::Screen::get_width() as f32;
        let char = engage::app::UnitInfo::get_instance().m_windows().get(0).m_unit_info_window_chara_model().m_chara();
        if !char.is_null(){
            let go = char.get_game_object();
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
                            if pixels.get((y * w) + (x_size2 + x0 - dx)).a < 0.01 { empty_right += 1; }
                            if pixels.get((y * w) + (x0 - x_size2 + dx)).a < 0.01 { empty_left += 1; }
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
                    let mut raw_face = vec![Color{r: 1.0, g: 1.0, b: 1.0, a: 1.0}; x_size * y_size];
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
                            let mut color = pixels.get(((y + y_min) * w) + (x + x_min)).get_gamma();
                            color.a = color.a * trans_alpha;
                            let idx = x + y * x_size;
                            raw_face[idx] = color;
                        }
                    }
                    let scaled = resize(&raw_face, x_size as i32, y_size as i32, 188, 74);
                    texture_cropped.resize_impl(188, 74);
                    for y in 0..74 {
                        for x in 0..188 { texture_cropped.set_pixel(x as i32, y as i32, scaled[x + y * 188]); }
                        texture_cropped.set_pixel(187, y as i32, Color{r: 0.0, g: 0.0, b: 0.0, a: 0.0});
                    }
                    texture_cropped.apply_3();
                    if let Some(file_path) = save_texture_png(texture_cropped, true) {
                        if assign_face {
                            let name = file_path.split("/").last().unwrap();
                            if let Some(unit) = UnitAssetMenuData::get_unit() {
                                if let Some((active, original, loaded)) = get_unit_face_keys(unit){
                                    let key = format!("G_Face_{}", active);
                                    if !engage::GameVariableManager::is_exist(key.as_str()) {
                                        let user_data = GameUserData::get_instance();
                                        let vars: GameVariable = user_data.get_variable();
                                        vars.entry_2(key.as_str(), name);
                                    }
                                    else { engage::GameVariableManager::set_string(key.as_str(), name); }
                                    let rect = engage::unity_engine::Rect{
                                        m_x_min: 0.0,
                                        m_y_min: 0.0,
                                        m_width: 188.0,
                                        m_height: 74.0,
                                    };
                                    texture_cropped.set_filter_mode(engage::unity_engine::FilterMode::trilinear());
                                    let sprite = Sprite::create_5(
                                        texture_cropped,
                                        rect,
                                        engage::unity_engine::Vector2{x: 0.5, y: 0.5},
                                        100.0,
                                        1u32,
                                        engage::unity_engine::SpriteMeshType::tight()
                                    );
                                    let face_thumb = engage::app::FaceThumbnail::s_face_thumb().m_cache_table();
                                    let (found, original) = face_thumb.try_get_value(original.as_str().into());
                                    if found && !original.is_null(){
                                        let (found2, alt) = face_thumb.try_get_value(loaded.as_str().into());
                                        if found2 && !alt.is_null() {
                                            if !original.equals(alt) { engage::unity_engine::Object_2::destroy_2(alt); }
                                        }
                                    }
                                    face_thumb.try_insert(active.as_str().into(), sprite, InsertionBehavior::overwrite_existing());
                                    face_thumb.try_insert(loaded.as_str().into(), sprite, InsertionBehavior::overwrite_existing());
                                    engage::app::GameMessage::create_key_wait(proc, format!("Assigned and saved face thumbnail to\n'{}'.", file_path.as_str()));
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
        else { message = "Unable to save face thumbnail to file.".to_string(); }
    }
    engage::app::GameMessage::create_key_wait(proc, message);
    engage::unity_engine::Object_2::destroy_2(texture_cropped);
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
            let mut v_sample: Vec<_> = vec![Color{r: 0.0, b: 0.0, g: 0.0, a: 0.0}; width * new_height];
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
                    let mut c = Color{r: 0.0, b: 0.0, g: 0.0, a: 0.0};
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
    let mut out: Vec<_> = vec![Color{r: 0.0, b: 0.0, g: 0.0, a: 0.0}; new_height * new_width];
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
            let mut c = Color{r: 0.0, b: 0.0, g: 0.0, a: 0.0};
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
pub fn save_texture_png(texture2d1: engage::unity_engine::Texture2D, is_face: bool) -> Option<String> {
    let data1 = engage::unity_engine::ImageConversion::encode_to_png(texture2d1);
    if let Some(unit) = UnitAssetMenuData::get_unit() {
        let name = unit.get_name();
        let path = if is_face { THUMB_DIR } else { CAPTURE_DIR };
        let file_path = crate::get_next_filename(path, &name.to_string(), "png");
        if let Ok(mut file) = std::fs::File::options().create(true).write(true).truncate(true).open(file_path.as_str()){
            let result = file.write_all(data1.as_slice());
            if result.is_ok(){ return Some(file_path); }
        }
    }
    None
}
/// Gets Dictionary Keys from Unit's Ascii name
/// - active (ascii name) used in game
/// - original (o_ + ascii name) the original sprite
/// - loaded (a_ + ascii_name) the replacement sprite
pub fn get_unit_face_keys(unit: engage::app::Unit) -> Option<(String, String, String)> {
    let ascii_name = unit.get_person().get_ascii_name();
    let mut active = ascii_name.to_string();
    let mut original = format!("o_{}", ascii_name);
    let mut loaded = format!("a_{}", ascii_name);

    if unit.get_person().index() == 1 && unit.m_edit().m_gender().value == 2 {
        active.push('W');
        loaded.push('W');
        original.push('W');
    }
    Some((active, original, loaded))
}
pub fn update_face(unit: engage::app::Unit, use_original: bool){
    if let Some((active, loaded, original)) = get_unit_face_keys(unit) {
        let face_thumb = engage::app::FaceThumbnail::s_face_thumb().m_cache_table();
        let (found, sprite) =
            if use_original { face_thumb.try_get_value(original.as_str().into()) }
            else { face_thumb.try_get_value(loaded.as_str().into()) };

        if found && !sprite.is_null() { face_thumb.try_insert(active.as_str().into(), sprite, InsertionBehavior::overwrite_existing()); }
    }
}
pub fn reset_faces(title: bool) {
    let face_thumb = engage::app::FaceThumbnail::s_face_thumb().m_cache_table();
    let s =
        face_thumb.iter()
            .filter(|(k, _)| !k.is_null())
            .filter(|(k, _v)| k.to_rust_string().starts_with("o_"))
            .map(|(k, v)| (k.to_rust_string(), v))
            .collect::<Vec<(String, Sprite)>>();
    s.iter().for_each(|(o, s)|{
        let active =  o.trim_start_matches("o_").to_string();
        let load = format!("a_{}", active);
        let (found, loaded) = face_thumb.try_get_value(load.as_str().into());
        if found {
            if !engage::unity_engine::Object_2::op_equality(loaded, *s){
                face_thumb.try_insert(load.as_str().into(), *s, InsertionBehavior::overwrite_existing());
                face_thumb.try_insert(active.as_str().into(), *s, InsertionBehavior::overwrite_existing());
                engage::unity_engine::Object_2::destroy_2(loaded);
            }
        }
    });
      if !UnitAssetMenuData::get().is_loaded && !title {
          UnitAssetMenuData::get().data.iter().for_each(|d|{
              let person = engage::app::PersonData::try_get_from_hash(d.person);
              if !person.is_null() {
                  let ascii_name = person.get_ascii_name();
                  let ascii = if d.flag & 16 == 0 { ascii_name.to_rust_string() } else { format!("{}W", ascii_name) };
                  load_png_to_by_ascii(&ascii, d.flag & 8 != 0);
              }
          });
      }

    return;
}
fn load_png_to_by_ascii(ascii: &String, use_sprite: bool) -> bool {
    let file_key = format!("G_Face_{}", ascii);
    if engage::GameVariableManager::is_exist(file_key.as_str()) {
        let file = engage::GameVariableManager::get_string(file_key.as_str()).to_rust_string();
        if !file.starts_with("---") && file.contains(".png") {
            let p = format!("{}{}", THUMB_DIR, file);
            let path = Path::new(p.as_str());
            if path.exists() {
                if let Some(mut file) = std::fs::read(path).ok().filter(|d| png_file_check(d)){
                    if let Some(sprite) = create_face_sprite(&mut file) {
                        let alt_key = format!("a_{}", ascii);
                        let table = engage::app::FaceThumbnail::s_face_thumb().m_cache_table();
                        if table.try_insert(alt_key.as_str().into(), sprite, InsertionBehavior::overwrite_existing()) {
                            if use_sprite {
                                table.try_insert(ascii.as_str().into(), sprite, InsertionBehavior::overwrite_existing());
                            }
                        }
                        return true;
                    }
                }
            }
            else { engage::GameVariableManager::set_string(file_key.as_str(), "---"); }
        }
    }
    false
}
pub fn create_face_sprite(data: &mut Vec<u8>) -> Option<engage::unity_engine::Sprite> {
    let data = unity::Array::from_slice(data.as_slice())?;
    let new_texture = engage::unity_engine::Texture2D::new_9(188, 74);
    if engage::unity_engine::ImageConversion::load_image_2(new_texture, data) {
        new_texture.set_filter_mode(engage::unity_engine::FilterMode::trilinear());
        let sprite =
            Sprite::create_5(
            new_texture,
            engage::unity_engine::Rect{ m_x_min: 0.0, m_y_min: 0.0, m_width: 188.0, m_height: 74.0, },
            engage::unity_engine::Vector2{x: 0.5, y: 0.5},
            100.0,
            1u32,
            engage::unity_engine::SpriteMeshType::tight()
        );
        if !sprite.is_null() {
            Some(sprite)
        }
        else { None }
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
fn get_screen_height(optional_method: unity::OptionalMethod) -> i32;

#[skyline::from_offset(0x2f8b920)]
fn get_screen_width(optional_method: unity::OptionalMethod) -> i32;