use engage::unit::{Gender};
use engage::gameicon::GameIcon;
use engage::menu::content::{AccessoryDetailInfoWindow, AccessoryEquipmentInfo};
use engage::menu::menu_item::accessory::AccessoryMenuItemContent;
use engage::mess::Mess;
use engage::unityengine::GameObject;
use unity::engine::Sprite;
use unity::engine::ui::IsImage;
use unity::system::Il2CppString;
use crate::{get_current_profile_name, get_outfit_data, AssetType, MenuText, MenuTextCommand, PlayerOutfitData, UnitAssetMenuData};
use crate::items::Profile;
use crate::menu::icons::CustomMenuIcon;

#[derive(PartialEq, Clone, Copy)]
pub enum EquipmentBoxMode {
    CurrentProfile,
    CurrentProfilePage(EquipmentBoxPage),
    ProfilePreview(Profile),
    LoadData(EquipmentBoxPage),
}
#[derive(PartialEq, Clone, Copy)]
pub enum EquipmentBoxPage {
    Flags,
    Assets,
    AccessoryAssets,
    AOCAnimations,
    RideMounts,
    Color(u8),
    Scaling(u8),
}
impl EquipmentBoxPage {
    pub fn get_next(self) -> EquipmentBoxPage {
        match self {
            EquipmentBoxPage::Flags => { EquipmentBoxPage::Assets }
            EquipmentBoxPage::Assets => { EquipmentBoxPage::AccessoryAssets  }
            EquipmentBoxPage::AccessoryAssets => { EquipmentBoxPage::AOCAnimations }
            EquipmentBoxPage::AOCAnimations => { EquipmentBoxPage::RideMounts  }
            EquipmentBoxPage::RideMounts => { EquipmentBoxPage::Color(0) }
            EquipmentBoxPage::Color(kind) => {
                if kind == 0 { EquipmentBoxPage::Color(4) }
                else { EquipmentBoxPage::Scaling(0) }
            }
            EquipmentBoxPage::Scaling(set) => {
                if set < 3 { EquipmentBoxPage::Scaling(1+set) }
                else { EquipmentBoxPage::Flags }
            }
        }
    }
    pub fn get_previous(self) -> EquipmentBoxPage {
        match self {
            EquipmentBoxPage::Flags => { EquipmentBoxPage::Scaling(3) }
            EquipmentBoxPage::Assets => { EquipmentBoxPage::Flags }
            EquipmentBoxPage::AccessoryAssets => { EquipmentBoxPage::Assets }
            EquipmentBoxPage::AOCAnimations => { EquipmentBoxPage::AccessoryAssets }
            EquipmentBoxPage::RideMounts => { EquipmentBoxPage::AOCAnimations }
            EquipmentBoxPage::Color(kind) => {
                if kind < 4 { EquipmentBoxPage::RideMounts } else { EquipmentBoxPage::Color(0) }
            }
            EquipmentBoxPage::Scaling(set) => {
                if set == 0 { EquipmentBoxPage::Color(4) }
                else { EquipmentBoxPage::Scaling(set - 1) }
            }
        }
    }
    pub fn get_preset_appearance(self, increase: bool) -> EquipmentBoxPage {
        let mut new = self;
        loop {
            new = if increase { new.get_next() } else { new.get_previous() };
            if new != EquipmentBoxPage::Flags { return  new; }
        }
    }
}

impl EquipmentBoxMode {
    pub fn set_profile(equipment: &mut AccessoryEquipmentInfo, profile: Option<Profile>) {
        let (name, flag) =
            profile.and_then(|v| UnitAssetMenuData::get_current_asset_data().map(|d| { (v.get_name(), d.profile[v.to_index()].flag) }))
                .unwrap_or_else(|| (get_current_profile_name(), UnitAssetMenuData::get_preview().preview_data.flag));
        if let Some(con) = equipment.menu_list[0].menu_item_content.as_mut() { set_icon_text_to_content(con, CustomMenuIcon::KeyItem.get_icon(), Some(name)); }
        Self::set_profile_flags(equipment, flag);
    }
    pub fn set_profile_flags(equipment: &mut AccessoryEquipmentInfo, flag: i32) {
        let engage =
            if flag & 6 == 2 { format!("{}: {}", MenuTextCommand::Engage, MenuTextCommand::on_off(false)) }
            else if flag & 6 == 4 { format!("{}: {}", MenuTextCommand::Engage, MenuTextCommand::Emblem) }
            else { format!("{}: {}", MenuTextCommand::Engage, MenuTextCommand::on_off(true)) }.into();

        set_content_data_slot(equipment, 1, CustomMenuIcon::EngageCommon.get_icon(), Some(engage));
        set_content_data_slot(equipment, 2, CustomMenuIcon::Star.get_icon(), Some(format!("{}: {}", MenuText::get_command(20), MenuTextCommand::on_off(flag & 1 != 0)).into()));
        set_content_data_slot(equipment, 3, CustomMenuIcon::StarBlank.get_icon(), Some(format!("{}: {}", MenuText::get_command(22), MenuTextCommand::on_off(flag & 64 != 0)).into()));
        set_content_data_slot(equipment, 4,  CustomMenuIcon::Gift.get_icon(), Some(format!("{}: {}", MenuText::get_command(23), MenuTextCommand::on_off(flag & 32 != 0)).into()));
        let (kind, icon) = if UnitAssetMenuData::get().is_shop_combat { ("MID_TUT_CATEGORY_TITLE_Battle", CustomMenuIcon::Weapon) } else { ("MID_SAVEDATA_SEQ_HUB", CustomMenuIcon::Day) };
        set_content_data_slot(equipment, 5, icon.get_icon(), Some(format!("Viewing: {}", Mess::get(kind)).into()));
    }
    pub fn set_data(equipment: &mut AccessoryEquipmentInfo, page: EquipmentBoxPage, data: Option<&PlayerOutfitData>) {
        if UnitAssetMenuData::is_photo_graph()  { return; }
        let db = get_outfit_data();
        let preview = UnitAssetMenuData::get_preview();
        let no_data = data.is_none();
        if let Some(data) = data.or(Some(&preview.preview_data)) {
            match page {
                EquipmentBoxPage::Flags => { Self::set_profile_flags(equipment, data.flag); }
                EquipmentBoxPage::Assets => {
                    let ubody = db.try_get_asset(AssetType::Body, data.ubody)
                        .or_else(|| db.try_get_asset(AssetType::Body, preview.original_assets[0]).filter(|_| no_data))
                        .map(|v| v.split_once("_").unwrap().1.into())
                        .unwrap_or(Mess::get_item_none2());

                    let obody = db.hashes.get_obody(data.ubody).map(|v| v.to_string().trim_start_matches("oBody_").into())
                        .or_else(|| db.hashes.o_body.get(&preview.original_assets[3]).filter(|_| no_data).map(|v| v.split_once("_").unwrap().1.into()))
                        .unwrap_or(Mess::get_item_none2());

                    set_content_data_slot(equipment, 1, CustomMenuIcon::Clothes.get_icon(), Some(format!("{} / {}", ubody, obody).into()));

                    let head = db.try_get_asset(AssetType::Head, data.uhead)
                        .or_else(|| db.try_get_asset(AssetType::Head, preview.original_assets[1]).filter(|_| no_data))
                        .map(|v| v.into())
                        .or_else(|| Some(Mess::get_item_none2()));

                    set_content_data_slot(equipment, 2, CustomMenuIcon::Head.get_icon(), head);

                    let hair = db.try_get_asset(AssetType::Hair, data.uhair)
                        .or_else(|| db.try_get_asset(AssetType::Hair, preview.original_assets[2]).filter(|_| no_data))
                        .map(|v| v.into())
                        .or_else(|| Some(Mess::get_item_none2()));

                    set_content_data_slot(equipment, 3, CustomMenuIcon::Hair.get_icon(), hair);

                    let rig = db.try_get_asset(AssetType::Rig, data.rig)
                        .or_else(|| db.try_get_asset(AssetType::Rig, preview.original_assets[15]).filter(|_| no_data))
                        .map(|v| v.into())
                        .or_else(|| Some(Mess::get_item_none2()));

                    set_content_data_slot(equipment, 4, CustomMenuIcon::Body.get_icon(), rig);

                    let ohead = db.hashes.get_ohair(data.uhair)
                        .or_else(|| db.hashes.o_hair.get(&preview.original_assets[4]).map(|v| v.into()).filter(|_| no_data))
                        .or_else(|| db.hashes.get_ohair(preview.original_assets[2]).map(|v| v.into()).filter(|_| no_data))
                        .or_else(|| db.hashes.get_ohair(preview.original_assets[1]).map(|v| v.into()).filter(|_| no_data))
                        .or_else(|| Some(Mess::get_item_none2()));

                    set_content_data_slot(equipment, 5, CustomMenuIcon::Head.get_icon(), ohead);
                }
                EquipmentBoxPage::AccessoryAssets => {
                    for x in 0..5 {
                        let name = db.try_get_asset(AssetType::Acc(x as u8), data.acc[x])
                            .or_else(|| db.try_get_asset(AssetType::Acc(x as u8), preview.original_assets[5 + x]).filter(|_| no_data))
                            .map(|v| v.into())
                            .or_else(|| Some(Mess::get_item_none2()));
                        let icon =
                            if x < 3 { GameIcon::try_get_system("Face") } else if x == 3 { GameIcon::try_get_system("EngCommon") } else { GameIcon::try_get_item("Byleth_AegisShield") };
                        set_content_data_slot(equipment, 1 + x, icon, name);
                    }
                }
                EquipmentBoxPage::RideMounts => {
                    for x in 0..5 {
                        let icon = GameIcon::try_get_efficacy(match x {
                            2 => { "Dragon" },
                            3 | 4 => { "Fly" },
                            _ => { "Horse" }
                        }, true);
                        set_content_data_slot(equipment, 1 + x, icon, db.try_get_asset(AssetType::Mount(x as u8), data.mount[x]).map(|v| v.into()).or_else(|| Some(Mess::get_item_none2())));
                    }
                }
                EquipmentBoxPage::AOCAnimations => {
                    let gender = db.get_dress_gender_hash(data.ubody).unwrap_or(if UnitAssetMenuData::get_current_dress_gender() == 2 { Gender::Female } else { Gender::Male });
                    for x in 0..4 {
                        let hash = if gender == Gender::Female { data.aoc_alt[x] } else { data.aoc[x] };
                        let aoc_name = db.try_get_asset(AssetType::AOC(x as u8), hash)
                            .or_else(|| db.try_get_asset(AssetType::AOC(x as u8), preview.original_assets[10 + x]).filter(|_| no_data))
                            .map(|v| v.into()).or_else(|| Some(Mess::get_item_none2()));

                        set_content_data_slot(equipment, 1 + x, GameIcon::try_get_system("SolaTail"), aoc_name);
                    }
                    let voice = db.try_get_asset(AssetType::Voice, data.voice)
                        .or_else(|| db.try_get_asset(AssetType::Voice, preview.original_assets[14]).filter(|_| no_data))
                        .map(|v| v.into()).or_else(|| Some(Mess::get_item_none2()));
                    set_content_data_slot(equipment, 5, GameIcon::try_get_system("TalkRelianceOutline"), voice);
                }
                EquipmentBoxPage::Color(kind) => {
                    let offset = if kind >= 4 { 4 } else { 0 } as usize;
                    let enable = data.flag & 1 != 0;
                    set_content_data_slot(equipment, 1, CustomMenuIcon::Star.get_icon(), Some(format!("{}: {}", MenuText::get_command(20), MenuTextCommand::on_off(enable)).into()));
                    for x in 0..4 {
                        let color = x + offset;
                        let color_str =
                            if data.colors[color].has_color() && (!no_data || enable) {
                                format!("{}: {}", MenuText::get_command(1140 + color as i32), data.colors[color])
                            }
                            else if no_data {
                                format!("{}: {}/{}/{}",
                                    MenuText::get_command(1140 + color as i32),
                                    preview.original_color[4 * color],
                                    preview.original_color[4 * color + 1],
                                    preview.original_color[4 * color + 2],
                                )
                            } else { format!("{}: --/--/--", MenuText::get_command(1140 + color as i32)) }.into();

                        set_content_data_slot(equipment, 2 + x, CustomMenuIcon::Star.get_icon(), Some(color_str));
                    }
                }
                EquipmentBoxPage::Scaling(set) => {
                    let enable = data.flag & 64 != 0;
                    set_content_data_slot(
                        equipment,
                        1,
                        CustomMenuIcon::StarBlank.get_icon(),
                        Some(format!("{}: {}", MenuText::get_command(22),MenuTextCommand::on_off(enable)).into())
                    );
                    for x in 0..4 {
                        let scale_index = set as usize * 4 + x;
                        let value = if data.scale[scale_index] > 0 && data.scale[scale_index] < 1000 { (data.scale[scale_index] as f32 / 100.0).to_string() }
                        else if no_data { (preview.original_scaling[scale_index] as f32 / 100.0).to_string() }
                        else { "--".to_string() };
                        set_content_data_slot(
                            equipment, 2 + x,
                            CustomMenuIcon::StarBlank.get_icon(),
                            Some(format!("{}: {}", MenuText::get_command(150 + scale_index as i32), value).into())
                        );
                    }
                }
            }
        }
    }
    pub fn change_equipment_box(self, equipment: &mut AccessoryEquipmentInfo) {
        if UnitAssetMenuData::is_photo_graph()   { return; }
        if let Some(con) = equipment.menu_list[0].menu_item_content.as_mut() {
            let profile_name = get_current_profile_name();
            set_icon_text_to_content(con, GameIcon::try_get_system("KeyItem"), Some(profile_name));
        }
        for x in 0..6 {
            equipment.menu_list[x].kind = x as i32;
            if let Some(con) = equipment.menu_list[x].menu_item_content.as_mut() {
                con.name_object.set_active(true);
                con.kind_icon.set_active(true);
            }
        };
        match self {
            Self::ProfilePreview(profile) => { Self::set_profile(equipment, Some(profile)); }
            Self::CurrentProfile => { Self::set_profile(equipment, None); }
            Self::CurrentProfilePage(page) => { Self::set_data(equipment, page, None); }
            Self::LoadData(page) => {
                let menu = UnitAssetMenuData::get();
                if let Some(data) = menu.loaded_data.get_selected_data() {
                    let name = data.get_filename().into();
                    if let Some(con) = equipment.menu_list[0].menu_item_content.as_mut() {
                        set_icon_text_to_content(con, CustomMenuIcon::KeyItem.get_icon(), Some(name));
                    }
                    Self::set_data(equipment, page, Some(&data.data));
                }
                else { Self::set_data(equipment, page, None); }
            }
        }
    }
    pub fn set_preset_appearance(self, appearance_index: i32) {
        if let Some(appearance) = get_outfit_data().dress.personal.get(appearance_index as usize) {
            let data = PlayerOutfitData::from_appearance(appearance);
            match self {
                Self::CurrentProfilePage(page)|Self::LoadData(page) => {
                    if let Some(equipment) = GameObject::find("EquipmentAcc").and_then(|go| go.get_component_by_type::<AccessoryEquipmentInfo>()) {
                        Self::set_data(equipment, page, Some(&data));
                        set_content_data_slot(equipment, 0, None, Some(appearance.get_name()));
                    }
                }
                _ => {}
            }

        }
    }
    pub fn update(self) {
        if UnitAssetMenuData::is_photo_graph()   { return; }
        if let Some(equipment) = GameObject::find("EquipmentAcc").and_then(|go| go.get_component_by_type::<AccessoryEquipmentInfo>()) {
            self.change_equipment_box(equipment);
        }
    }
    pub fn change_cursor(equipment: &mut AccessoryEquipmentInfo, kind: Option<i32>) {
        if UnitAssetMenuData::is_photo_graph()  { return; }
        if let Some(k) = kind { equipment.show_cursor3(k); }
        else { equipment.hide_cursor(); }
    }
    pub fn set_cursor(kind: Option<i32>){
        if UnitAssetMenuData::is_photo_graph() { return; }
        if let Some(equipment) = GameObject::find("EquipmentAcc").and_then(|go| go.get_component_by_type::<AccessoryEquipmentInfo>()) {
            Self::change_cursor(equipment, kind);
        }
    }
}
pub fn set_content_data_slot(equipment: &mut AccessoryEquipmentInfo, slot: usize, icon: Option<&'static mut Sprite>, name: Option<&'static Il2CppString>) {
    if let Some(con) = equipment.menu_list[slot].menu_item_content.as_mut() {
        set_icon_text_to_content(con, icon, name);
    }
}
pub fn set_icon_text_to_content(content: &mut AccessoryMenuItemContent, icon: Option<&'static mut Sprite>, name: Option<&'static Il2CppString>) {
    if let Some(icon) = icon { content.kind_icon_image.set_sprite2(icon); }
    if let Some(name) = name.as_ref() { content.name_text.set_text(name, true); }
}

pub fn set_detail_box(name: Option<&Il2CppString>, help: Option<&Il2CppString>, body: Option<&Il2CppString>, sprite: Option<&'static mut Sprite>) {
    if UnitAssetMenuData::is_photo_graph()  { return; }
    if let Some(detail_box) = GameObject::find("WdwAccHelp").and_then(|go| go.get_component_by_type::<AccessoryDetailInfoWindow>()) {
        if let Some(help) = help.as_ref() { detail_box.message.set_text(help, true); }
        if let Some(name) = name.as_ref() { detail_box.accessory_name.set_text(name, false); }
        if let Some(body) = body.as_ref() { detail_box.body_parts[0].text.set_text(body, false); }
        if let Some(sprite1) = sprite { detail_box.body_parts[0].image.set_sprite2(sprite1); }
    }
}