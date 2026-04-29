use engage::{
    unit::Gender,
    gameuserdata::GameUserData,
    map::mind::MapMind,
    mess::Mess,
    random::Random,
    sequence::hubaccessory::room::HubAccessoryRoom,
    unitinfo::UnitInfo,
    util::get_singleton_proc_instance,
};
use engage::combat::CharacterAppearance;
use engage::gamedata::assettable::AssetTableResult;
use engage::gamesound::{GameSound, GameSoundFadeSpeedType};
use crate::{get_outfit_data, left_right_enclose, new_asset_table_accessory, EquipmentBoxPage, MenuTextCommand, Mount, UnitAssetMenuData, ACC_LOC, V_EVENTS};
use crate::anim::AnimData;
use crate::data::room::hub_room_set_by_result;
use crate::localize::MenuText;
use crate::room::ReloadType;
use super::*;

#[derive(PartialEq, Copy, Clone)]
pub enum AssetType {
    Body,
    Head,
    Hair,
    AOC(u8),
    Acc(u8),
    Mount(u8),
    Voice,
    ColorPreset(u8),
    Rig,
}
impl AssetType {
    pub fn to_index(&self) -> i32 {
        match self {
            AssetType::Body => 0,
            AssetType::Head => 1,
            AssetType::Hair => 2,
            AssetType::Voice => 3,
            AssetType::Rig => 4,
            AssetType::Acc(k) => 10 + *k as i32,
            AssetType::Mount(k) => 20 + *k as i32,
            AssetType::AOC(k) => 30 + *k as i32,
            AssetType::ColorPreset(k) =>  40 + *k as i32,
        }
    }
    pub fn from_rel_index(index: i32) -> Option<AssetType> {
        if index >= 50 { None }
        else {
            Some(
                match index {
                    0 => AssetType::Body,
                    1 => AssetType::Head,
                    2 => AssetType::Hair,
                    3 => AssetType::Voice,
                    4 => AssetType::Rig,
                    10..15 => AssetType::Acc(index as u8 - 10),
                    20..25 => AssetType::Mount(index as u8 - 20),
                    30..34 => AssetType::AOC(index as u8 - 30),
                    40..50 => AssetType::ColorPreset(index as u8 - 40),
                    _ => unreachable!(),
                }
            )
        }
    }
    pub fn update_preview(&self, menu_item: &CustomAssetMenuItem, equipment_box: bool, model: bool) {
        if equipment_box {
            self.get_equipment_box_type(menu_item).update();
            match self {
                AssetType::Body => { EquipmentBoxMode::set_cursor(Some(1)); }
                AssetType::Head => { EquipmentBoxMode::set_cursor(Some(2)) }
                AssetType::Hair => { EquipmentBoxMode::set_cursor(Some(3)) }
                AssetType::Rig => { EquipmentBoxMode::set_cursor(Some(4)); }
                AssetType::Acc(k) => { EquipmentBoxMode::set_cursor(Some(*k as i32 + 1)); }
                AssetType::Mount(kind) => { EquipmentBoxMode::set_cursor(Some(*kind as i32 + 1)); }
                AssetType::AOC(kind) => { EquipmentBoxMode::set_cursor(Some(*kind as i32 + 1)); }
                AssetType::Voice => { EquipmentBoxMode::set_cursor(Some(5)); }
                AssetType::ColorPreset(kind) => {
                    EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Color(*kind)).update();
                    let color_kind = *kind as i32;
                    let cursor_pos = if color_kind < 4 { color_kind + 2 } else { color_kind - 2 };
                    EquipmentBoxMode::set_cursor(Some(cursor_pos));
                }
            }
        }
        if model {
            let mut reload_type = ReloadType::ForcedUpdate;
            println!("Updating");
            let menu_index =  menu_item.menu.menu_kind.to_index();
            let is_personal = menu_index == 10;
            let is_engaged = menu_index == 16 || menu_index == 17;
            let result = if is_engaged {
                let pid = if menu_index == 16 { "PID_青リュール_男性" } else { "PID_青リュール_女性" };
                println!("IS ENGAGED");
                let result = AssetTableResult::get_from_pid(2, pid, CharacterAppearance::get_constions(None));
                result
            }
            else { UnitAssetMenuData::get_result() };

            let photo = UnitAssetMenuData::is_photo_graph();
            if photo && !is_personal {
                let hash = menu_item.hash;
                let preview = UnitAssetMenuData::get_preview();
                match self {
                    AssetType::ColorPreset(kind) => {
                        for x in 0..3 { preview.preview_data.colors[*kind as usize].values[x] = ((hash >> x*8) & 255) as u8; }

                        reload_type = ReloadType::ColorScale;
                    }
                    AssetType::Body => {
                        preview.preview_data.ubody = hash;
                        if !is_personal { reload_type = ReloadType::Dress; }
                    }
                    AssetType::Hair => { preview.preview_data.uhair = hash; }
                    AssetType::Head => { preview.preview_data.uhead = hash; }
                    AssetType::Mount(kind) => { preview.preview_data.mount[*kind as usize] = hash; }
                    AssetType::Acc(kind) => { preview.preview_data.acc[*kind as usize] = hash; }
                    AssetType::AOC(kind) => {
                        if get_outfit_data().get_aoc_gender_hash(*kind as i32, hash) == Some(Gender::Male) { preview.preview_data.aoc[*kind as usize] = hash; }
                        else { preview.preview_data.aoc_alt[*kind as usize] = hash; }
                    }
                    AssetType::Rig => { preview.preview_data.rig = hash; },
                    _ => {}
                }
                preview.preview_data.set_result(result, 2, false, false);
                hub_room_set_by_result(Some(result), reload_type);
                return;
            }
            result.ride_dress_model = None;
            result.ride_model = None;
            result.left_hand = "null".into();
            result.right_hand = "null".into();
            let db = get_outfit_data();
            let asset = db.try_get_asset(*self, menu_item.hash);
            match self {
                AssetType::Body => {
                    if let Some(asset) = asset {
                        result.dress_model = asset.as_str().into();
                        result.body_anim = Some(
                            if db.get_dress_gender(result.dress_model) == Gender::Male { "AOC_Hub_Hum0M" }
                            else { "AOC_Hub_Hum0F" }.into()
                        );
                        if UnitAssetMenuData::get_preview().update_dress_gender {
                            UnitAssetMenuData::get_preview().update_dress_gender = false;
                            reload_type = ReloadType::ForcedUpdate;
                        }
                        else if !is_personal && !is_engaged { reload_type = ReloadType::Dress; }
                    }
                }
                AssetType::Rig => {
                    if let Some(asset) = asset {
                        result.body_model = asset.as_str().into();
                        if !is_personal { reload_type = ReloadType::Body; }
                    }
                }
                AssetType::Head => {
                    if let Some(asset) = asset {
                        result.head_model = asset.as_str().into();
                        if !is_personal { reload_type = ReloadType::Head; }
                    }
                }
                AssetType::Hair => {
                    if let Some(asset) = asset {
                        crate::apply_hair(asset, result);
                        result.replace(2);
                        if !is_personal { reload_type = ReloadType::Hair; }
                    }
                }
                AssetType::Acc(kind) => {
                    if let Some(asset) = asset {
                        if asset.contains("Msc0AT") { result.left_hand = asset.into(); }
                        else {
                            let acc_locator = ACC_LOC[*kind as usize];
                            result.commit_accessory(new_asset_table_accessory(asset.as_str(), acc_locator));
                            result.replace(2);
                            EquipmentBoxMode::set_cursor(Some(*kind as i32 + 1));
                            if !is_personal { reload_type = ReloadType::Accessories(*kind as usize); }
                        }
                    }
                }
                AssetType::Mount(kind) => {
                    if GameUserData::get_sequence() == 3 {
                        if let Some(unit) = MapMind::get_unit() { unit.reload_actor(); }
                    }
                    if let Some(asset) = asset {
                        result.body_anims.clear();
                        let dress = db.get_dress_gender(result.dress_model);
                        let gender = if db.get_dress_gender(result.dress_model) == Gender::Female { "F" } else { "M" };
                        result.ride_dress_model = Some(asset.into());
                        result.ride_model = Some(Mount::from_i32(1+*kind as i32).get_default_asset(true).into());
                        match kind {
                            0 => {
                                let anim = format!("Cav0B{}-No1_c000_N", gender);
                                result.body_anims.add(format!("Com0B{}-No1_c000_N", gender).into());
                                result.body_anims.add(anim.as_str().into());
                                result.body_anim = Some(anim.into());
                            }
                            1 => {
                                let anim = format!("Cav2C{}-No1_c000_N", gender);
                                result.body_anims.add(format!("Com0B{}-No1_c000_N", gender).into());
                                result.body_anims.add(anim.as_str().into());
                                result.body_anim = Some(anim.into());
                            }
                            2 => {
                                let anim = format!("Wng2D{}-No1_c000_N", gender);
                                result.body_anims.add(anim.as_str().into());
                                result.body_anim = Some(anim.into());
                            }
                            3 => {
                                if dress == Gender::Male { result.dress_model = "uBody_Wng0EF_c000".into(); }
                                let anim = "Wng0EF-No1_c000_N";
                                result.body_anims.add(anim.into());
                                result.body_anim = Some(anim.into());
                            }
                            4 => {
                                let anim = format!("Wng1F{}-No1_c000_N", gender);
                                result.body_anims.add(anim.as_str().into());
                                result.body_anim = Some(anim.into());
                            }
                            _ => { result.body_anims.add(format!("Com0A{}-No1_c000_N", gender).into()); }
                        }
                        hub_room_set_by_result(Some(result), ReloadType::Mount);
                    }
                    return;
                }
                AssetType::AOC(_) => {
                    if let Some(asset) = asset {
                        AnimData::remove(result, true, true);
                        result.left_hand = "null".into();
                        result.right_hand = "null".into();
                        result.info_anims = None;
                        result.talk_anims = None;
                        result.demo_anims = None;
                        result.hub_anims = None;
                        result.body_anim = Some(asset.into());
                        AnimData::remove(result, true, true);
                        return hub_room_set_by_result(Some(result), ReloadType::ForcedUpdate);
                    }
                }
                AssetType::Voice => { return; }
                AssetType::ColorPreset(kind) => {
                    let menu_data = UnitAssetMenuData::get_preview();
                    let mut current_color = [0; 3];
                    let color_kind = *kind as usize;
                    for x in 0..3 { current_color[x] = menu_data.original_color[4*color_kind+x]; }
                    let selected_color: [u8; 3] = [
                        (menu_item.hash & 255) as u8,
                        ((menu_item.hash >> 8) & 255) as u8,
                        ((menu_item.hash >> 16) & 255) as u8
                    ];
                    result.unity_colors[color_kind].r = selected_color[0] as f32 / 255.0;
                    result.unity_colors[color_kind].g = selected_color[1] as f32 / 255.0;
                    result.unity_colors[color_kind].b = selected_color[2] as f32 / 255.0;
                    hub_room_set_by_result(Some(result), ReloadType::ColorScale);
                    return;
                }
            }
            if UnitAssetMenuData::is_unit_info() && *self != AssetType::Body { result.body_anim = result.hub_anims; }
            hub_room_set_by_result(Some(result), reload_type);
        }
    }
}
impl CustomMenuItem for AssetType {
    fn get_icon(&self, menu_item: &CustomAssetMenuItem) -> CustomMenuIcon {
        match self {
            Self::Body => {
                match menu_item.menu.menu_kind {
                    ShopBody((2, _)) => CustomMenuIcon::Engaged(menu_item.index as u8),
                    _ => CustomMenuIcon::Clothes,
                }
            },
            Self::Rig => CustomMenuIcon::Body,
            Self::Head => CustomMenuIcon::Head,
            Self::Hair => CustomMenuIcon::Hair,
            Self::Voice => CustomMenuIcon::Talk,
            Self::AOC(_) => CustomMenuIcon::SolaTail,
            Self::Acc(kind) => {
                match kind {
                    3 => CustomMenuIcon::EngageCommon,
                    4 => CustomMenuIcon::Shield,
                    _ => CustomMenuIcon::AccFace
                }
            }
            Self::Mount(kind) => { CustomMenuIcon::Mount(*kind) }
            Self::ColorPreset(_) => { CustomMenuIcon::Star }
        }
    }
    fn get_equipment_box_type(&self, _menu_item: &CustomAssetMenuItem) -> EquipmentBoxMode {
        match self {
            Self::Body|Self::Head|Self::Hair|Self::Rig => EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Assets),
            Self::Acc(_) => EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::AccessoryAssets),
            Self::Voice|Self::AOC(_) => EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::AOCAnimations),
            Self::ColorPreset(kind) =>  EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Color(*kind)),
            Self::Mount(_) =>  EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::RideMounts),
        }
    }
    fn get_name(&self, menu_item: &CustomAssetMenuItem) -> &'static Il2CppString {
        match self {
            Self::Body => { if menu_item.hash == UnitAssetMenuData::get_preview().preview_data.break_body { return format!("{} [B]", menu_item.name).into(); } }
            _ => {}
        }
        menu_item.name
    }
    fn get_detail_box_name(&self, menu_item: &CustomAssetMenuItem) -> Option<&'static Il2CppString> { Some(menu_item.name) }
    fn get_help(&self, menu_item: &CustomAssetMenuItem) -> &'static Il2CppString {
        let db = get_outfit_data();
        if let Some(mode2) = db.try_get_asset(*self, menu_item.hash){
            let idx = menu_item.menu.menu_kind.to_index() + 1000;
            let help = MenuText::get_help(idx).unwrap_or(format!("MenuHelp #{}", idx).into());
            match self {
                Self::Body => {
                    let mode1 = db.hashes.get_obody(menu_item.hash).unwrap_or(Mess::get_item_none());
                    if UnitAssetMenuData::get().god_mode {
                        let idx = menu_item.menu.menu_kind.to_index();
                        if idx == 16 { format!("Male Engaged Outfit\nCombat: {} / Map: {}", mode2, mode1) }
                        else if idx == 16 { format!("Female Engaged Outfit\nCombat: {} / Map: {}", mode2, mode1) }
                        else { format!("Combat: {}\nMap: {}", mode2, mode1) }.into()
                    }
                    else {
                        format!("Combat: {} / Map: {}\n{}Set for break [Experimental: {}].",
                            mode2, mode1, Mess::create_sprite_tag_str(2, "X"),
                            MenuTextCommand::on_off( UnitAssetMenuData::get_flag() & 32 != 0 ) ).into()
                    }
                }
                Self::Rig => format!("Combat Rig: {}", mode2).into(),
                Self::Head => format!("Combat Head: {}", mode2).into(),
                Self::Hair => {
                    let mode1 = db.hashes.get_ohair(menu_item.hash).unwrap_or(Mess::get_item_none());
                    format!("Combat: {}\nMap: {}", mode2, mode1).into()
                }
                Self::Mount(_) => {
                    let mode1 = db.hashes.get_mount_obody(menu_item.hash).unwrap_or(Mess::get_item_none());
                    format!("Combat: {}\nMap: {}", mode2, mode1).into()
                }
                Self::AOC(_) => { format!("{}\nAnimation Set: {}", help, mode2).into() },
                Self::Acc(_) => format!("{}\nAsset: {}", help, mode2).into(),
                Self::Voice => format!("Voice Set: {}", mode2).into(),
                _ => { "ColorPreset".into() }
            }
        }
        else {
            match self {
                Self::ColorPreset(kind) => {
                    let menu_data = UnitAssetMenuData::get_preview();
                    let mut current_color = [0; 3];
                    let color_kind = *kind as usize;
                    for x in 0..3 { current_color[x] = menu_data.original_color[4 * color_kind + x]; }
                    let selected_color: [u8; 3] = [(menu_item.hash & 255) as u8, ((menu_item.hash >> 8) & 255) as u8, ((menu_item.hash >> 16) & 255) as u8];
                    let mut changed = false;
                    for x in 0..3 { changed |= selected_color[x] != current_color[x]; }
                    let color_enabled = if UnitAssetMenuData::get_flag() & 1 == 0 {  " <color=\"yellow\">[Not Active].</color>" } else { " "};
                    let preset_color_str = format!("{}/{}/{}", selected_color[0], selected_color[1], selected_color[2]);
                    if changed { format!("{}: {}{}", MenuTextCommand::Confirm.get_with_sys_sprite("A"), preset_color_str, color_enabled).into() }
                    else if menu_item.decided { format!("{}: {}{}",MenuTextCommand::A.to_right(MenuTextCommand::Reset), preset_color_str, color_enabled).into() }
                    else { format!("{}{}", MenuTextCommand::Select.get_with_value(preset_color_str), color_enabled).into() }
                }
                _ => { Mess::get_item_none() }
            }
        }
    }
    fn get_body(&self, menu_item: &CustomAssetMenuItem) -> &'static Il2CppString {
        let personal = menu_item.menu.menu_kind == Personal;
        let idx = self.to_index() + 50;
        match self {
            AssetType::ColorPreset(kind) => { format!("{} (Preset)", MenuText::get_command(1140+*kind as i32)).into() }
            AssetType::AOC(kind) => {
                let db = get_outfit_data();
                let mut body = format!("{} ({})", MenuText::get_command(idx), if db.get_aoc_gender_hash(*kind as i32, menu_item.hash) == Some(Gender::Male) { "Male" } else { "Female" });
                if !personal {
                    body.push_str(&format!(" [{}/4]", *kind +1).as_str());
                    left_right_enclose(&body)
                }
                else { body.into() }
            },
            AssetType::Acc(kind) => make_body_asset_body_label(MenuText::get_command(idx), if personal { 0 } else { 5 }, *kind as i32),
            AssetType::Mount(kind) => {
                if GameUserData::get_sequence() == 3 { MenuText::get_command(idx) }
                else { make_body_asset_body_label(MenuText::get_command(idx), if personal { 0 } else { 5 }, *kind as i32) }
            },
            AssetType::Head => Head.get_name(menu_item),
            AssetType::Hair => Hair.get_name(menu_item),
            AssetType::Rig => "Model Rig".into(),
            _ => { MenuTextCommand::Personal.get() }
        }
    }
    fn a_call(&self, menu_item: &mut CustomAssetMenuItem) -> BasicMenuResult {
        let preview = UnitAssetMenuData::get_preview();
        let hash = if menu_item.decided { 0 } else { menu_item.hash };
        match self {
            AssetType::ColorPreset(kind) => {
                for x in 0..3 { preview.preview_data.colors[*kind as usize].values[x] = ((hash >> x*8) & 255) as u8; }
                let index = menu_item.index;
                menu_item.menu.full_menu_item_list.iter_mut().for_each(|x|{ x.set_decided(x.index == index && hash != 0); });
            }
            AssetType::Body => {
                let idx = menu_item.menu.menu_kind.to_index();
                if idx == 16 { preview.preview_data.mount[0] = hash; }
                else if idx == 17 { preview.preview_data.mount[1] = hash; }
                else { preview.preview_data.ubody = hash; }
            }
            AssetType::Hair => { preview.preview_data.uhair = hash; }
            AssetType::Head => { preview.preview_data.uhead = hash; }
            AssetType::Mount(kind) => { preview.preview_data.mount[*kind as usize] = hash; }
            AssetType::Acc(kind) => { preview.preview_data.acc[*kind as usize] = hash; }
            AssetType::AOC(kind) => {
                if get_outfit_data().get_aoc_gender_hash(*kind as i32, hash) == Some(Gender::Male) { preview.preview_data.aoc[*kind as usize] = hash; }
                else { preview.preview_data.aoc_alt[*kind as usize] = hash; }
            }
            AssetType::Rig => { preview.preview_data.rig = hash; },
            AssetType::Voice => {
                preview.preview_data.voice = hash;
                let db = get_outfit_data();
                if let Some(voice) = db.hashes.voice.get(&hash) {
                    let rng = Random::get_system();
                    if let Some(char) = get_singleton_proc_instance::<HubAccessoryRoom>().and_then(|v| v.character.as_ref())
                        .or_else(|| UnitInfo::get_instance().map(|v| &v.windows[0].unit_info_window_chara_model.char))
                    {
                        let event =
                            if voice.contains("Shop") {
                                if rng.get_value(2) == 0 { format!("V_{}_Thanks", voice) }
                                else { format!("V_{}_Tutorial", voice) }
                            }
                            else { V_EVENTS[rng.get_value(V_EVENTS.len() as i32) as usize].to_string() };
                        GameSound::stop_all_voice(GameSoundFadeSpeedType::Fast);
                        GameSound::person_voice2(voice.into(), None, Some(event.into()), Some(char));
                    }
                }
            }
        }
        menu_item.menu.full_menu_item_list.iter_mut().for_each(|x|{
            match x.menu_kind {
                Asset(_) => {
                    let decided = hash == x.hash;
                    x.set_decided(decided);
                    if decided {
                        println!("Name: {}", x.name);
                    }
                    x.rebuild_text();
                }
                _ => {}
            }
        });
        self.get_equipment_box_type(menu_item).update();
        menu_item.rebuild_text();
        BasicMenuResult::se_decide()
    }
}
fn make_body_asset_body_label(label: &'static Il2CppString, page_count: i32, page: i32) -> &'static Il2CppString {
    if page_count == 0 { label } else { left_right_enclose(&format!("{} [{}/{}]", label, page+1, page_count)) }
}