use engage::{
    system::collections::generic::IList_1Methods,
    app::ISingletonProcInst_1Methods,
    app::{AssetTable_Modes, AssetTable_Result, IAccessoryMenuItemMethods, IAssetTable_Result, IAssetTable_ResultMethods, IBasicMenu, IBasicMenuItem, IBasicMenuItemMethods},
    List_1Ext
};
use engage::app::{HubAccessoryRoom, IGameUserDataMethods, IHubAccessoryRoom, IProcInstMethods, IRandom_2Methods, ISingletonClass_1Methods, IUnitInfo, IUnitInfoWindowCharaModel, IUnitInfo_Window, UnitInfo};
use unity::Cast;
use crate::{get_outfit_data, left_right_enclose, EquipmentBoxPage, MenuTextCommand, UnitAssetMenuData, V_EVENTS, data::room::hub_room_set_by_result, localize::MenuText, room::ReloadType, set_color_by_i32};
use super::*;
use std::io::Write;

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
    pub fn default_icon(self) -> CustomMenuIcon {
        match self {
            AssetType::Body => CustomMenuIcon::Clothes,
            AssetType::Head => CustomMenuIcon::Head,
            AssetType::Hair => CustomMenuIcon::Hair,
            AssetType::AOC(_) => CustomMenuIcon::SolaTail,
            AssetType::Acc(x) => {
                if x < 3 { CustomMenuIcon::AccFace }
                else if x == 3 { CustomMenuIcon::EngageCommon }
                else { CustomMenuIcon::Shield }
            }
            AssetType::Mount(k) => CustomMenuIcon::Mount(k),
            AssetType::Voice => CustomMenuIcon::Talk,
            AssetType::ColorPreset(_) => CustomMenuIcon::Color,
            AssetType::Rig => CustomMenuIcon::Body,
        }
    }
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
                    40..51 => AssetType::ColorPreset(index as u8 - 40),
                    _ => unreachable!(),
                }
            )
        }
    }
    pub fn update_model(&self, menu_item: CustomAssetMenuItem3) {
        let mut reload_type = ReloadType::ForcedUpdate;
        let menu_index = menu_item.get_asset_menu().menu_kind().to_index();
        let is_engaged = menu_index == 16 || menu_index == 17;
        let result = if is_engaged {
            let pid = if menu_index == 16 { "PID_青リュール_男性" } else { "PID_青リュール_女性" };
            let result = AssetTable_Result::get_from_pid(AssetTable_Modes::combat(), pid, engage::combat::CharacterAppearance::get_constions(crate::get_default_asset_conditions()));
            result
        }
        else { UnitAssetMenuData::get_result() };
        /*
        let photo = UnitAssetMenuData::is_photo_graph();
        if photo {
            let hash = menu_item.value();
            let preview = UnitAssetMenuData::get_preview();
            match self {
                AssetType::ColorPreset(kind) => {
                    for x in 0..3 { preview.preview_data.colors[*kind as usize].values[x] = ((hash >> x*8) & 255) as u8; }
                    reload_type = ReloadType::ColorScale;
                }
                AssetType::Body => {
                    preview.preview_data.ubody = hash;
                    reload_type = ReloadType::Dress;
                }
                AssetType::Hair => { preview.preview_data.uhair = hash; }
                AssetType::Head => { preview.preview_data.uhead = hash; }
                AssetType::Mount(kind) => { preview.preview_data.mount[*kind as usize] = hash; }
                AssetType::Acc(kind) => { preview.preview_data.acc[*kind as usize] = hash; }
                AssetType::AOC(kind) => {
                    if get_outfit_data().get_aoc_gender_hash(*kind as i32, hash) == Some(engage::app::Gender::male()) { preview.preview_data.aoc[*kind as usize] = hash; }
                    else { preview.preview_data.aoc_alt[*kind as usize] = hash; }
                }
                AssetType::Rig => { preview.preview_data.rig = hash; },
                _ => {}
            }
            preview.preview_data.set_result(result, 2, false, false);
            hub_room_set_by_result(Some(result), reload_type);
            return;
        }
        */
        result.set_ride_model("");
        result.set_ride_dress_model("");
        result.set_left_hand("null");
        result.set_right_hand("null");
        match self {
            AssetType::Voice => { return; }
            AssetType::ColorPreset(kind) => {
                let v = menu_item.value();
                let k = *kind % 16;
                let selected_color: [u8; 3] = [(v & 255) as u8, ((v >> 8) & 255) as u8, ((v>> 16) & 255) as u8];
                let menu_data = UnitAssetMenuData::get_preview();
                if k < 8 { set_color_by_i32(result, k as usize, v); }
                for x in 0..3 { menu_data.color_preview[4*(k as usize) + x] =  selected_color[x]; }
                hub_room_set_by_result(Some(result), ReloadType::ColorScale);
                return;
            }
            AssetType::AOC(_) => {
                UnitAssetMenuData::get_preview().preview_asset = Some((self.clone(), menu_item.value()));
                hub_room_set_by_result(Some(result), ReloadType::AOC);
                return;
            }
            _ => { UnitAssetMenuData::get_preview().preview_asset = Some((self.clone(), menu_item.value())); }
        }
        if UnitAssetMenuData::is_unit_info() && *self != AssetType::Body { result.set_body_anim(result.m_hub_anim()); }
        hub_room_set_by_result(Some(result), ReloadType::All);
    }
    pub fn update_box(&self, menu_item: CustomAssetMenuItem3){
        self.get_equipment_box_type(menu_item).update();
        match self {
            AssetType::Body => { EquipmentBoxMode::set_cursor(Some(1)); }
            AssetType::Rig => { EquipmentBoxMode::set_cursor(Some(2)); }
            AssetType::Head => { EquipmentBoxMode::set_cursor(Some(1)) }
            AssetType::Hair => { EquipmentBoxMode::set_cursor(Some(1)) }
            AssetType::Acc(k) => { EquipmentBoxMode::set_cursor(Some(*k as i32 + 1)); }
            AssetType::Mount(kind) => { EquipmentBoxMode::set_cursor(Some(*kind as i32 + 1)); }
            AssetType::AOC(kind) => { EquipmentBoxMode::set_cursor(Some(*kind as i32 + 1)); }
            AssetType::Voice => { EquipmentBoxMode::set_cursor(Some(5)); }
            AssetType::ColorPreset(kind) => {
                let v2 = menu_item.value2();
                if v2 == 14 { EquipmentBoxMode::set_cursor(Some(4)); }
                else {
                    let k = *kind;
                    EquipmentBoxMode::set_color_cursor(Some(k as i32));
                }
            }
        }
    }
}
impl CustomMenuItem for AssetType {
    fn get_icon(&self, menu_item: CustomAssetMenuItem3) -> CustomMenuIcon {
        match self {
            Self::Body => {
                let menu = menu_item.get_asset_menu();
                match menu.menu_kind() {
                    ShopBody((2, _)) => CustomMenuIcon::Engaged(menu_item.get_index() as u8),
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
            Self::ColorPreset(_) => { CustomMenuIcon::Color }
        }
    }
    fn get_equipment_box_type(&self, item: CustomAssetMenuItem3) -> EquipmentBoxMode {
        match self {
            Self::Body|Self::Rig => EquipmentBoxMode::Body,
            Self::Head => EquipmentBoxMode::Head,
            Self::Hair => EquipmentBoxMode::Hair,
            Self::Acc(_) => EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::AccessoryAssets),
            Self::Voice|Self::AOC(_) => EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::AOCAnimations(item.female())),
            Self::ColorPreset(kind) => {
                let v2 = item.value2();
                if v2 == 14 { EquipmentBoxMode::Hair }
                else { EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Color(*kind)) }
            }
            Self::Mount(_) =>  EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::RideMounts),
        }
    }
    fn get_name(&self, menu_item: CustomAssetMenuItem3) -> unity::Il2CppString {
        menu_item.m_name()
    }
    fn get_detail_box_name(&self, menu_item: CustomAssetMenuItem3) -> Option<unity::Il2CppString> {
        Some(menu_item.m_name())
    }
    fn get_help(&self, menu_item: CustomAssetMenuItem3) -> unity::Il2CppString {
        let db = get_outfit_data();
        let menu_idx = menu_item.get_asset_menu().menu_kind().to_index();
        if let Some(mode2) = db.try_get_asset(*self, menu_item.value()){
            let idx = menu_idx + 1000;
            let help = MenuText::get_help(idx).unwrap_or(format!("MenuHelp #{}", idx).into());
            match self {
                Self::Body => {
                    let mode1 = db.hashes.get_obody(menu_item.value()).unwrap_or("------".into());
                    if UnitAssetMenuData::get().god_mode {
                        let idx = menu_idx;
                        if idx == 16 { format!("Male Engaged Outfit\nCombat: {} / Map: {}", mode2, mode1) }
                        else if idx == 17 { format!("Female Engaged Outfit\nCombat: {} / Map: {}", mode2, mode1) }
                        else { format!("Combat/Map: {} / Map: {}\n{}", mode2, mode1, MenuTextCommand::LeftRight.insert_right("Change Page")) }.into()
                    }
                    else {
                        format!("Combat/Map {} / {}\n{} ", mode2, mode1, MenuTextCommand::LeftRight.insert_right("Change Page")).into()
                    }
                }
                Self::Rig => format!("Combat Rig: {}", mode2).into(),
                Self::Head => format!("Combat Head: {}", mode2).into(),
                Self::Hair => {
                    let mode1 = db.hashes.get_ohair(menu_item.value()).unwrap_or("------".into());
                    format!("Combat: {}\nMap: {}", mode2, mode1).into()
                }
                Self::Mount(_) => {
                    let mode1 = db.hashes.get_mount_obody(menu_item.value()).unwrap_or("------".into());
                    format!("Combat: {}\nMap: {}", mode2, mode1).into()
                }
                Self::AOC(_) => format!("{}\nAnimation Set: {}", help, mode2).into(),
                Self::Acc(_) => format!("{}\nAsset: {}", help, mode2).into(),
                Self::Voice => format!("Voice Set: {}", mode2).into(),
                _ => { "ColorPreset".into() }
            }
        }
        else {
            match self {
                Self::ColorPreset(_) => {
                    let selected_color: [u8; 3] = [(menu_item.value() & 255) as u8, ((menu_item.value() >> 8) & 255) as u8, ((menu_item.value() >> 16) & 255) as u8];
                    let preset_color_str = format!("{}/{}/{}", selected_color[0], selected_color[1], selected_color[2]);
                    format!("Replacing color: {}\n{}: {}",
                        MenuText::get_command(1140 + menu_item.value2()),
                        MenuTextCommand::Confirm.get_with_sys_sprite("A"),
                        preset_color_str
                    ).into()
                }
                _ => {"------".into() }
            }
        }
    }
    fn get_body(&self, menu_item: CustomAssetMenuItem3) -> unity::Il2CppString {
        let idx = self.to_index() + 50;
        match self {
            AssetType::ColorPreset(kind) => { format!("{} (Preset)", MenuText::get_command(1140+*kind as i32)).into() }
            AssetType::AOC(kind) => {
                let db = get_outfit_data();
                let mut body = format!("{} ({})", MenuText::get_command(idx), if db.get_aoc_gender_hash(*kind as i32, menu_item.value()) == Some(engage::app::Gender::male()) { "Male" } else { "Female" });
                body.push_str(&format!(" [{}/5]", *kind +1).as_str());
                left_right_enclose(&body)
            },
            AssetType::Acc(kind) => make_body_asset_body_label(MenuText::get_command(idx), 5, *kind as i32),
            AssetType::Mount(kind) => {
                if engage::app::GameUserData::get_instance().get_sequence().value == 3 { MenuText::get_command(idx) }
                else { make_body_asset_body_label(MenuText::get_command(idx), 5, *kind as i32) }
            },
            AssetType::Head => make_body_asset_body_label(Head.get_name(menu_item), 2, 0),
            AssetType::Hair => Hair.get_name(menu_item),
            AssetType::Rig => "Model Rig".into(),
            _ => { MenuTextCommand::Personal.get() }
        }
    }
    fn a_call(&self, menu_item: CustomAssetMenuItem3) -> BasicMenu_Result {
        let preview = UnitAssetMenuData::get_preview();
        let hash = if menu_item.get_m_decided() { 0 } else { menu_item.value() };
        match self {
            AssetType::ColorPreset(kind) => {
                for x in 0..3 { preview.preview_data.colors[*kind as usize].values[x] = ((hash >> x*8) & 255) as u8; }
                menu_item.get_asset_menu().b_call();
                return BasicMenu_Result::se_decide();
            }
            AssetType::Body => {
                let idx = menu_item.get_asset_menu().menu_kind().to_index();
                if idx == 16 { preview.preview_data.mount[0] = hash; }
                else if idx == 17 { preview.preview_data.mount[1] = hash; }
                else { preview.preview_data.ubody = hash; }
            }
            AssetType::Hair => {
                preview.update = 1;
                preview.preview_data.uhair = hash;
            }
            AssetType::Head => {
                preview.update = 2;
                preview.preview_data.uhead = hash;
            }
            AssetType::Mount(kind) => { preview.preview_data.mount[*kind as usize] = hash; }
            AssetType::Acc(kind) => { preview.preview_data.acc[*kind as usize] = hash; }
            AssetType::AOC(kind) => {
                if get_outfit_data().get_aoc_gender_hash(*kind as i32, hash) == Some(engage::app::Gender::male()) { preview.preview_data.aoc[*kind as usize] = hash; }
                else { preview.preview_data.aoc_alt[*kind as usize] = hash; }
            }
            AssetType::Rig => { preview.preview_data.rig = hash; },
            AssetType::Voice => {
                preview.preview_data.voice = hash;
                let db = get_outfit_data();
                if let Some(voice) = db.hashes.voice.get(&hash) {
                    let rng = engage::app::Random_2::get_system();
                    let room = HubAccessoryRoom::get_instance();
                    let character =
                    if !room.is_null() { Some(room.m_character()) }
                    else { Some(UnitInfo::get_instance().m_windows().get(0).m_unit_info_window_chara_model().m_chara()) };
                    if let Some(character) = character.filter(|c| !c.is_null()){
                        let event =
                            if voice.contains("Shop") {
                                if rng.get_value_2(2) == 0 { format!("V_{}_Thanks", voice) }
                                else { format!("V_{}_Tutorial", voice) }
                            }
                            else { V_EVENTS[rng.get_value_2(V_EVENTS.len() as i32) as usize].to_string() };
                        engage::app::GameSound::stop_all_voice(engage::app::GameSound_FadeSpeedType::fast());
                        engage::app::GameSound::person_voice_2(
                            voice.as_str(),
                            unity::Il2CppString::null(),
                            event,
                            character
                        );
                    }
                }
            }
        }
        menu_item.get_asset_menu().m_full_menu_item_list().iter().for_each(|v|{ 
            let i = unsafe { v.cast::<CustomAssetMenuItem3>() };
            let decided = i.value() == hash;
            i.set_m_decided(decided);
            i.rebuild_text();
        });
        self.get_equipment_box_type(menu_item).update();
        menu_item.rebuild_text();
        BasicMenu_Result::se_decide()
    }
}
pub fn make_body_asset_body_label(label: unity::Il2CppString, page_count: i32, page: i32) -> unity::Il2CppString {
    if page_count == 0 { label } else { left_right_enclose(&format!("{} [{}/{}]", label, page+1, page_count)) }
}