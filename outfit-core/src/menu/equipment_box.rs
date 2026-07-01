use engage::{
    List_1Ext,
    app::{
        AccessoryData_Kinds,
        BasicMenuItem,IBasicMenuItem, IBasicMenuItemContentMethods, IBasicMenuItemMethods,
        accessorydetailinfowindow::*, accessoryequipmentinfo::*, accessorymenuitem::*, accessorymenuitemcontent::*,
    },
    prelude::List_1,
    system::collections::generic::IList_1Methods,
    unity_engine::{
        gameobject::*,
        IComponentMethods, IObject_2Methods, ITransformMethods, IRectTransformMethods,
        ui::{IGraphicMethods, IImageMethods},
    },
    tm_pro::ITMP_TextMethods,
};
use unity::{Cast, FromIlInstance, IlNull};
use crate::{get_current_profile_name, get_outfit_data, AssetType, MenuText, MenuTextCommand, PlayerOutfitData, UnitAssetMenuData, items::Profile, menu::icons::CustomMenuIcon, FACIAL_STATES};
const BLANK: &'static str = "------";
pub fn build_equipment_window(this: engage::app::AccessoryEquipmentInfo, is_room: bool) {
    let content = this.m_content_object();
    if content.is_null() { return; }
    let transform = content.get_transform();
    let child_1 = transform.get_child(0).get_game_object();
    for i in 0..4 {
        let obj = engage::unity_engine::Object_2::instantiate_3(child_1);
        obj.set_name(format!("Acc{}",i+6));
        if let Some(go) = obj.try_cast::<GameObject>() {
            let go_t = go.get_transform();
            go_t.set_parent(transform);
        }
    }
    let list = List_1::<BasicMenuItem>::new();
    this.set_m_menu_item_list(list);
    let count = transform.get_child_count();
    for i in 0..count {
        let item = engage::app::AccessoryMenuItem::instantiate().unwrap();
        IBasicMenuItemMethods::ctor(item);
        item.set_m_index(i);
        item.set_m_accessory_kind(AccessoryData_Kinds{value: i});
        let child_transform = transform.get_child(i);
        let item_content = child_transform.get_game_object().get_component::<AccessoryMenuItemContent>();
        let rect = item_content.get_rect_transform();
        let mut size = rect.get_size_delta();
        size.x += 40.0;
        crate::change_rect_transform_in_children_size(child_transform, "Name", 40.0, 0.0);
        rect.set_size_delta(size);
        item_content.build(item);
        list.add(BasicMenuItem::from(item));
    }
    if is_room { EquipmentBoxMode::HubPreview.change_equipment_box(this); }
}


#[derive(PartialEq, Clone, Copy)]
pub enum EquipmentBoxMode {
    HubPreview,
    CurrentProfile,
    CurrentProfilePage(EquipmentBoxPage),
    ProfilePreview(Profile),
    LoadData(EquipmentBoxPage),
    Body,
    Hair,
    Head,
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
            EquipmentBoxPage::Color(kind) => { if kind < 8 { EquipmentBoxPage::Color(8) } else { EquipmentBoxPage::Scaling(0) } }
            EquipmentBoxPage::Scaling(set) => {
                if set == 0 { EquipmentBoxPage::Scaling(1) }
                else { EquipmentBoxPage::Flags }
            }
        }
    }
    pub fn get_previous(self) -> EquipmentBoxPage {
        match self {
            EquipmentBoxPage::Flags => { EquipmentBoxPage::Scaling(1) }
            EquipmentBoxPage::Assets => { EquipmentBoxPage::Flags }
            EquipmentBoxPage::AccessoryAssets => { EquipmentBoxPage::Assets }
            EquipmentBoxPage::AOCAnimations => { EquipmentBoxPage::AccessoryAssets }
            EquipmentBoxPage::RideMounts => { EquipmentBoxPage::AOCAnimations }
            EquipmentBoxPage::Color(kind) => { if kind < 8 { EquipmentBoxPage::RideMounts } else { EquipmentBoxPage::Color(0) } }
            EquipmentBoxPage::Scaling(set) => { if set == 0 { EquipmentBoxPage::Color(8) } else { EquipmentBoxPage::Scaling(0) } }
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
    pub fn set_open(open: bool) {
        let go = GameObject::find("EquipmentAcc");
        if !go.is_null() {
            if let Some(equip) = get_equipment_box(){ if open { equip.open() } else { equip.close() } }
        }
    }
    pub fn set_rows(equipment: engage::app::AccessoryEquipmentInfo, row: i32) {
        let content_transform = equipment.m_content_object().get_transform();
        let child_count = content_transform.get_child_count();
        for i in 0..child_count {
            let child = content_transform.get_child(i);
            child.get_game_object().set_active(i < row);
        }
    }
    pub fn set_profile(equipment: engage::app::AccessoryEquipmentInfo, profile: Option<Profile>) {
        let (name, flag) =
            profile.and_then(|v| UnitAssetMenuData::get_current_asset_data().map(|d| { (v.get_name(), d.profile[v.to_index()].flag) }))
                .unwrap_or_else(|| (get_current_profile_name(), UnitAssetMenuData::get_preview().preview_data.flag));
        if let Some(con) = get_content(equipment, 0) { set_icon_text_to_content(con, CustomMenuIcon::KeyItem.get_icon(), Some(name)); }
        Self::set_profile_flags(equipment, flag);
    }
    pub fn set_profile_flags(equipment: engage::app::AccessoryEquipmentInfo, flag: i32) {
        Self::set_rows(equipment, 5);
        let engage =
            if flag & 6 == 2 { format!("{}: {}", MenuTextCommand::Engage, MenuTextCommand::on_off(false)) }
            else if flag & 6 == 4 { format!("{}: {}", MenuTextCommand::Engage, MenuTextCommand::Emblem) }
            else { format!("{}: {}", MenuTextCommand::Engage, MenuTextCommand::on_off(true)) }.into();

        set_content_data_slot(equipment, 1, CustomMenuIcon::EngageCommon.get_icon(), Some(engage));
        set_content_data_slot(equipment, 2, CustomMenuIcon::Body.get_icon(), Some(format!("{}: {}", MenuText::get_command(25), MenuTextCommand::on_off(flag & 8 != 0)).into()));
        set_content_data_slot(equipment, 3,  CustomMenuIcon::Gift.get_icon(), Some(format!("{}: {}", MenuText::get_command(23), MenuTextCommand::on_off(UnitAssetMenuData::get_person_flag() & 8 != 0)).into()));
        set_content_data_slot(equipment, 4, CustomMenuIcon::SolaTail.get_icon(), Some(format!("Expression: {}", FACIAL_STATES[UnitAssetMenuData::get().facial].0).into()));
    }
    pub fn set_viewing_mode(equipment: engage::app::AccessoryEquipmentInfo, slot: usize) {
        let (kind, icon) = if UnitAssetMenuData::get().is_shop_combat { ("MID_TUT_CATEGORY_TITLE_Battle", CustomMenuIcon::Weapon) } else { ("MID_SAVEDATA_SEQ_HUB", CustomMenuIcon::Day) };
        set_content_data_slot(equipment, slot, icon.get_icon(), Some(format!("Viewing: {}", engage::app::Mess::get(kind)).into()));
    }
    pub fn set_expression(equipment: engage::app::AccessoryEquipmentInfo, slot: usize) {
        set_content_data_slot(equipment, slot, CustomMenuIcon::SolaTail.get_icon(), Some(format!("Expression: {}", FACIAL_STATES[UnitAssetMenuData::get().facial].0).into()));
    }
    pub fn set_profile_name(equipment: engage::app::AccessoryEquipmentInfo, profile: Option<Profile>) {
        let name = profile.and_then(|v| UnitAssetMenuData::get_current_asset_data().map(|d| v.get_name()))
            .unwrap_or_else(|| get_current_profile_name());

        let name = format!("{} [Preview: {}]", name, engage::app::Mess::get(if UnitAssetMenuData::get().is_shop_combat { "MID_TUT_CATEGORY_TITLE_Battle" } else { "MID_SAVEDATA_SEQ_HUB" }));
        set_content_data_slot(equipment, 0, CustomMenuIcon::KeyItem.get_icon(), Some(name.as_str().into()))
    }
    pub fn set_asset(equipment: engage::app::AccessoryEquipmentInfo, slot: usize, kind: AssetType, data: Option<&PlayerOutfitData>) {
        let db = get_outfit_data();
        let preview = UnitAssetMenuData::get_preview();
        match kind {
            AssetType::ColorPreset(k) => {
                let k = k % 16;
                let color = data.as_ref().map(|d| d.get_asset_hash(kind))
                    .or_else(|| Some(preview.preview_data.get_asset_hash(kind)))
                    .unwrap_or(0);
                let color_str =
                    if color == 0 { "--/--/--".to_string() }
                    else { format!("{} / {} / {}", color & 255, (color >> 8) & 255, (color >> 16) & 255) };

                let text = format!("{}: {}", MenuText::get_command(1140 + k as i32), color_str);
                set_content_data_slot(equipment, slot, None, Some(text.as_str().into()));
                set_content_icon_color(equipment, slot, (k as i32) % 16, Some(color));
            }
            _ => {
                let h =
                data.as_ref().and_then(|d|
                    db.try_get_asset(kind, d.get_asset_hash(kind)).map(|v| unity::Il2CppString::from(v.as_str()))
                        .or_else(|| Some(BLANK.into()))
                ).or_else(||
                    db.try_get_asset(kind, preview.preview_data.get_asset_hash(kind)).map(|v| unity::Il2CppString::from(v.as_str()))
                        .or_else(|| db.try_get_asset(kind, preview.get_original_asset_hash(kind)).map(|v| unity::Il2CppString::from(v.as_str())))
                        .or_else(|| Some(BLANK.into()))
                );
                set_content_data_slot(equipment, slot, kind.default_icon().get_icon(), h);
            }
        }
    }
    pub fn set_data(equipment: engage::app::AccessoryEquipmentInfo, page: EquipmentBoxPage, data: Option<&PlayerOutfitData>) {
        if UnitAssetMenuData::is_photo_graph() { return; }
        let preview = UnitAssetMenuData::get_preview();
        match page {
            EquipmentBoxPage::Flags => {
                let flag = data.map(|p| p.flag).unwrap_or(preview.preview_data.flag);
                Self::set_profile_flags(equipment, flag);
            }
            EquipmentBoxPage::Assets => {
                Self::set_rows(equipment, 6);
                Self::set_asset(equipment, 1, AssetType::Body, data);
                Self::set_asset(equipment, 2, AssetType::Head, data);
                Self::set_asset(equipment, 3, AssetType::Hair, data);
                Self::set_asset(equipment, 4, AssetType::Rig, data);
                Self::set_asset(equipment, 5, AssetType::Voice, data);
                let obody = "-------";
                /*
                db.hashes.get_obody(data.ubody).map(|v| v.to_string().as_str())
                    .or_else(|| db.hashes.o_body.get(&preview.original_assets[3]).filter(|_| no_data).map(|v| v.split_once("_").unwrap().1))
                    .unwrap_or("------".into());
                let ohead = db.hashes.get_ohair(data.uhair)
                    .or_else(|| db.hashes.o_hair.get(&preview.original_assets[4]).map(|v| v.as_str().into()).filter(|_| no_data))
                    .or_else(|| db.hashes.get_ohair(preview.original_assets[2]).filter(|_| no_data))
                    .or_else(|| db.hashes.get_ohair(preview.original_assets[1]).filter(|_| no_data))
                    .or_else(|| Some("------".into()));
                 */
            }
            EquipmentBoxPage::AccessoryAssets => {
                Self::set_rows(equipment, 6);
                for x in 0..5 { Self::set_asset(equipment, x+1, AssetType::Acc(x as u8), data); }
            }
            EquipmentBoxPage::RideMounts => {
                Self::set_rows(equipment, 6);
                for x in 0..5 { Self::set_asset(equipment, x+1, AssetType::Mount(x as u8), data); }
            }
            EquipmentBoxPage::AOCAnimations => {
                Self::set_rows(equipment, 5);
                for x in 0..4 { Self::set_asset(equipment, x+1, AssetType::AOC(x as u8), data); }
                /*
                let gender = db.get_dress_gender_hash(data.ubody).unwrap_or(
                    if UnitAssetMenuData::get_current_dress_gender() == 2 {
                        engage::app::Gender::female() } else { engage::app::Gender::male() }
                );
                for x in 0..4 {
                    let hash = if gender == engage::app::Gender::female() { data.aoc_alt[x] } else { data.aoc[x] };
                    let aoc_name =
                        db.try_get_asset(AssetType::AOC(x as u8), hash)
                        .or_else(|| db.try_get_asset(AssetType::AOC(x as u8), preview.original_assets[10 + x]).filter(|_| no_data))
                        .map(|v| v.as_str().into()).or_else(|| Some("------".into()));

                    set_content_data_slot(equipment, 1 + x, CustomMenuIcon::SolaTail.get_icon(), aoc_name);
                }

                 */
            }
            EquipmentBoxPage::Color(kind) => {
                let k = kind % 16;
                let asset_colors = k < 8;
                let len = if asset_colors { 8 } else { 6 };
                Self::set_rows(equipment, len+1);
                for x in 0..len  {
                    let kind = if asset_colors { x as u8 } else { 8 + x as u8 };
                    let slot = x + 1;
                    Self::set_asset(equipment, slot as usize, AssetType::ColorPreset(kind), data);
                }
            }
            EquipmentBoxPage::Scaling(set) => {
                Self::set_rows(equipment, 9);
                let d = data.as_ref().map(|d| d.scale.clone()).unwrap_or(preview.preview_data.scale.clone());
                for x in 0..8 {
                    let scale_index = set as usize * 8 + x;
                    let value = if d[scale_index] > 0 && d[scale_index] < 1000 { (d[scale_index] as f32 / 100.0).to_string() }
                    else if data.is_none() { (preview.original_scaling[scale_index] as f32 / 100.0).to_string() }
                    else { "--".to_string() };
                    set_content_data_slot(
                        equipment, 1 + x,
                        CustomMenuIcon::StarBlank.get_icon(),
                        Some(format!("{}: {}", MenuText::get_command(150 + scale_index as i32), value).into())
                    );
                }
            }
        }
    }
    pub fn change_equipment_box(self, equipment: engage::app::AccessoryEquipmentInfo) {
        if UnitAssetMenuData::is_photo_graph()   { return; }
        if let Some(con) = get_content(equipment, 0) {
            let profile_name = get_current_profile_name();
            set_icon_text_to_content(con, CustomMenuIcon::KeyItem.get_icon(), Some(profile_name));
        }
        for x in 0..6 {
            if let Some(con) = get_content(equipment, x){
                con.m_kind_icon_object().set_active(true);
                con.m_name_object().set_active(true);
            }
        }
        match self {
            Self::ProfilePreview(profile) => {
                Self::set_profile(equipment, Some(profile));
                Self::set_profile_name(equipment, Some(profile));
            }
            Self::CurrentProfile => {
                Self::set_profile(equipment, None);
                Self::set_profile_name(equipment, None);
            }
            Self::CurrentProfilePage(page) => {
                Self::set_data(equipment, page, None);
                Self::set_profile_name(equipment, None);
            }
            Self::LoadData(page) => {
                let menu = UnitAssetMenuData::get();
                if let Some(data) = menu.loaded_data.get_selected_data() {
                    let name = data.get_filename().into();
                    if let Some(con) = get_content(equipment, 0) {
                        set_icon_text_to_content(con, CustomMenuIcon::KeyItem.get_icon(), Some(name));
                    }
                    Self::set_data(equipment, page, Some(&data.data));
                }
                else {
                    Self::set_data(equipment, page, None);
                    Self::set_profile_name(equipment, None);
                }
            }
            Self::Body => {
                Self::set_rows(equipment, 7);
                Self::set_asset(equipment, 1, AssetType::Body, None);
                Self::set_asset(equipment, 2, AssetType::Rig, None);
                Self::set_asset(equipment, 3, AssetType::ColorPreset(4), None);
                Self::set_asset(equipment, 4, AssetType::ColorPreset(5), None);
                Self::set_asset(equipment, 5, AssetType::ColorPreset(6), None);
                Self::set_asset(equipment, 6, AssetType::ColorPreset(7), None);
            }
            Self::Hair => {
                Self::set_rows(equipment, 5);
                Self::set_asset(equipment, 1, AssetType::Hair, None);
                Self::set_asset(equipment, 2, AssetType::ColorPreset(0), None);
                Self::set_asset(equipment, 3, AssetType::ColorPreset(1), None);
                Self::set_asset(equipment, 4, AssetType::ColorPreset(14), None);
                Self::set_profile_name(equipment, None);
            }
            Self::Head => {
                Self::set_rows(equipment, 3);
                Self::set_asset(equipment, 1, AssetType::Head, None);
                Self::set_asset(equipment, 2, AssetType::ColorPreset(2), None);
                Self::set_profile_name(equipment, None);
            }
            Self::HubPreview => {
                Self::set_rows(equipment, 7);
                Self::set_asset(equipment, 0, AssetType::Body, None);
                Self::set_asset(equipment, 1, AssetType::Head, None);
                Self::set_asset(equipment, 2, AssetType::Hair, None);
                for x in 0..4 { Self::set_asset(equipment, 3+x, AssetType::Acc(x as u8), None); }
            }
        }
    }
    pub fn set_preset_appearance(self, appearance_index: i32) {
        if let Some(appearance) = get_outfit_data().dress.personal.get(appearance_index as usize) {
            let data = PlayerOutfitData::from_appearance(appearance);
            match self {
                Self::CurrentProfilePage(page)|Self::LoadData(page) => {
                    if let Some(equipment) = get_equipment_box(){
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
        if let Some(equipment) = get_equipment_box(){ self.change_equipment_box(equipment); }
    }
    pub fn change_cursor(equipment: engage::app::AccessoryEquipmentInfo, kind: Option<i32>) {
        if UnitAssetMenuData::is_photo_graph()  { return; }
        if let Some(k) = kind { equipment.show_cursor_3(AccessoryData_Kinds{value: k}); }
        else { equipment.hide_cursor(); }
    }
    pub fn set_cursor(kind: Option<i32>){
        if UnitAssetMenuData::is_photo_graph() { return; }
        if let Some(equipment) = get_equipment_box() { Self::change_cursor(equipment, kind); }
    }
    pub fn set_color_cursor(color_kind: Option<i32>){
        if UnitAssetMenuData::is_photo_graph() { return; }
        if let Some(equipment) = get_equipment_box() { 
            let pos = color_kind.map(|v|{
                match v {
                    14 => 4,
                    _ => (v % 8) + 1,
                }
            });
            Self::change_cursor(equipment, pos);
        }
    }
}
pub fn get_equipment_box() -> Option<engage::app::AccessoryEquipmentInfo> {
    let go =  GameObject::find("EquipmentAcc");
    if !go.is_null() {
        let equip = go.get_component::<engage::app::AccessoryEquipmentInfo>();
        if !equip.is_null() { Some(equip) } else { None }
    }
    else { None }
}
pub fn get_content(equipment: engage::app::AccessoryEquipmentInfo, slot: i32) -> Option<AccessoryMenuItemContent> {
    let item = equipment.m_menu_item_list().get(slot);
    if !item.is_null() {
        let content = unsafe { item.get_menu_item_content().cast::<AccessoryMenuItemContent>() };
        return if !content.is_null() {Some(content) } else { None }
    }
    None
}
pub fn set_content_data_slot(equipment: AccessoryEquipmentInfo, slot: usize, icon: Option<engage::unity_engine::Sprite>, name: Option<unity::Il2CppString>) {
    let menu_item = equipment.m_menu_item_list().get(slot as i32);
    if !menu_item.is_null() {
        let content = unsafe { menu_item.get_menu_item_content().cast::<AccessoryMenuItemContent>() };
        if !content.is_null() { set_icon_text_to_content(content, icon, name); }
    }
}
pub fn set_icon_text_to_content(content: AccessoryMenuItemContent, icon: Option<engage::unity_engine::Sprite>, name: Option<unity::Il2CppString>) {
    if let Some(icon) = icon {
        content.m_kind_icon_image().set_color(engage::unity_engine::Color{r: 1.0, g: 1.0, b: 1.0, a: 1.0});
        content.m_kind_icon_object().set_active(true);
        content.m_kind_icon_image().set_sprite(icon);
    }
    else { content.m_kind_icon_object().set_active(false); }
    if let Some(name) = name { content.m_name_text().set_text_2(name, true); }
}
pub fn set_content_icon_color(equipment: AccessoryEquipmentInfo, slot: usize, kind: i32, color: Option<i32>) {
    let item = equipment.m_menu_item_list().get(slot as i32);
    if !item.is_null() {
        if let Some(content) = item.get_menu_item_content().try_cast::<AccessoryMenuItemContent>() {
            content.m_kind_icon_object().set_active(true);
            set_icon_to_color(content, kind, color);
        }
    }
}
pub fn set_icon_to_color(content: AccessoryMenuItemContent, kind: i32, color: Option<i32>) {
    let preview = UnitAssetMenuData::get_preview();
    if let Some(c) = color.filter(|c| *c > 0) {
        content.m_kind_icon_object().set_active(true);
        content.m_kind_icon_image().set_sprite(engage::unity_engine::Sprite::null());
        let r = (c & 255) as f32 / 255.0;
        let g = ((c >> 8) & 255) as f32 / 255.0;
        let b = ((c >> 16) & 255) as f32 / 255.0;
        content.m_kind_icon_image().set_color(engage::unity_engine::Color{r, g, b, a: 1.0});
    }
    else {
        if preview.preview_data.colors[kind as usize].has_color() {
            content.m_kind_icon_object().set_active(true);
            content.m_kind_icon_image().set_sprite(engage::unity_engine::Sprite::null());
            let c = UnitAssetMenuData::get_preview().preview_data.colors[kind as usize].get_f32();
            content.m_kind_icon_image().set_color(engage::unity_engine::Color{r: c[0], g: c[1], b: c[2], a: 1.0});
        }
        else {
            let i = 4*kind as usize;
            if preview.original_color[i] > 0 || preview.original_color[i+1] > 0 || preview.original_color[i+2] > 0 {
                content.m_kind_icon_image().set_sprite(engage::unity_engine::Sprite::null());
                content.m_kind_icon_object().set_active(true);
                content.m_kind_icon_image().set_color(
                    engage::unity_engine::Color{
                        r: preview.original_color[i] as f32 / 255.0 ,
                        g: preview.original_color[i+1] as f32 / 255.0 ,
                        b: preview.original_color[i+2] as f32 / 255.0 ,
                        a: 1.0
                    }
                );
            }
            else { content.m_kind_icon_object().set_active(false); }
        }
    }

}
pub fn set_detail_box(name: Option<unity::Il2CppString>, help: Option<unity::Il2CppString>, body: Option<unity::Il2CppString>, sprite: Option<engage::unity_engine::Sprite>) {
    //if UnitAssetMenuData::is_photo_graph()  { return; }
    let helpwdw = GameObject::find("WdwAccHelp");
    if !helpwdw.is_null() {
        let detail = helpwdw.get_component::<engage::app::AccessoryDetailInfoWindow>();
        if !detail.is_null() {
            if let Some(help) = help { detail.m_message().set_text_2(help, true); }
            if let Some(name) = name { detail.m_accessory_name().set_text_2(name, true); }
            if let Some(body) = body { detail.m_body_parts().get(0).m_text().set_text_2(body, true); }
            if let Some(sprite) = sprite {
                detail.m_body_parts().get(0).m_image().set_sprite(sprite);
            }
        }
    }
}