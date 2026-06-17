use engage_il2cpp::{
    BasicMenuItemExt,
    List_1Ext,
    app::{AccessoryData_Kinds, BasicMenuItem, IAccessoryDetailInfoWindow, IAccessoryDetailInfoWindow_BodyParts, IAccessoryEquipmentInfo, IAccessoryEquipmentInfoMethods, IAccessoryMenuItemContent, IAccessoryMenuItemMethods, IBasicMenuItem, IBasicMenuItemContentMethods, IBasicMenuItemMethods},
    prelude::List_1,
    system::collections::generic::IList_1Methods,
    tm_pro::ITMP_Text,
    unity_engine::{GameObject, IComponentMethods, IGameObjectMethods, IObject_2Methods, ITransformMethods},
    unity_engine::ui::{IGraphic, IGraphicMethods, IImageMethods}
};
use unity2::{Cast, FromIlInstance, IlNull};
use crate::{get_current_profile_name, get_outfit_data, AssetType, MenuText, MenuTextCommand, PlayerOutfitData, UnitAssetMenuData, items::Profile, menu::icons::CustomMenuIcon, FACIAL_STATES};

pub fn build_equipment_window(this: engage_il2cpp::app::AccessoryEquipmentInfo) {
    let content = this.m_content_object();
    if content.is_null() { return; }
    let transform = content.get_transform();
    let child_1 = transform.get_child(0).get_game_object();
    for i in 0..2 {
        let obj = engage_il2cpp::unity_engine::Object_2::instantiate_3(child_1);
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
        let item = engage_il2cpp::app::AccessoryMenuItem::instantiate().unwrap();
        IBasicMenuItemMethods::ctor(item);
        item.set_m_index(i);
        item.set_m_accessory_kind(AccessoryData_Kinds{value: i});
        let child_transform = transform.get_child(i);
        let item_content = child_transform.get_game_object().get_component::<engage_il2cpp::app::AccessoryMenuItemContent>();
        item_content.build(item);
        list.add(BasicMenuItem::from(item));
    }
}


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
    pub fn set_open(open: bool) {
        let go = GameObject::find("EquipmentAcc");
        if !go.is_null() {
            if let Some(equip) = get_equipment_box(){ if open { equip.open() } else { equip.close() } }
        }
    }
    pub fn set_profile(equipment: engage_il2cpp::app::AccessoryEquipmentInfo, profile: Option<Profile>) {
        let (name, flag) =
            profile.and_then(|v| UnitAssetMenuData::get_current_asset_data().map(|d| { (v.get_name(), d.profile[v.to_index()].flag) }))
                .unwrap_or_else(|| (get_current_profile_name(), UnitAssetMenuData::get_preview().preview_data.flag));
        if let Some(con) = get_content(equipment, 0) { set_icon_text_to_content(con, CustomMenuIcon::KeyItem.get_icon(), Some(name)); }
        Self::set_profile_flags(equipment, flag);
    }
    pub fn set_profile_flags(equipment: engage_il2cpp::app::AccessoryEquipmentInfo, flag: i32) {
        let engage =
            if flag & 6 == 2 { format!("{}: {}", MenuTextCommand::Engage, MenuTextCommand::on_off(false)) }
            else if flag & 6 == 4 { format!("{}: {}", MenuTextCommand::Engage, MenuTextCommand::Emblem) }
            else { format!("{}: {}", MenuTextCommand::Engage, MenuTextCommand::on_off(true)) }.into();

        set_content_data_slot(equipment, 1, CustomMenuIcon::EngageCommon.get_icon(), Some(engage));
        set_content_data_slot(equipment, 2, CustomMenuIcon::Body.get_icon(), Some(format!("{}: {}", MenuText::get_command(25), MenuTextCommand::on_off(flag & 8 != 0)).into()));
        set_content_data_slot(equipment, 3,  CustomMenuIcon::Gift.get_icon(), Some(format!("{}: {}", MenuText::get_command(23), MenuTextCommand::on_off(UnitAssetMenuData::get_person_flag() & 8 != 0)).into()));
        set_content_data_slot(equipment, 4, CustomMenuIcon::SolaTail.get_icon(), Some(format!("Expression: {}", FACIAL_STATES[UnitAssetMenuData::get().facial].0).into()));
        let (kind, icon) = if UnitAssetMenuData::get().is_shop_combat { ("MID_TUT_CATEGORY_TITLE_Battle", CustomMenuIcon::Weapon) } else { ("MID_SAVEDATA_SEQ_HUB", CustomMenuIcon::Day) };
        set_content_data_slot(equipment, 5, icon.get_icon(), Some(format!("Viewing: {}", engage_il2cpp::app::Mess::get(kind)).into()));
    }
    pub fn set_data(equipment: engage_il2cpp::app::AccessoryEquipmentInfo, page: EquipmentBoxPage, data: Option<&PlayerOutfitData>) {
        if UnitAssetMenuData::is_photo_graph()  { return; }
        let db = get_outfit_data();
        let preview = UnitAssetMenuData::get_preview();
        let no_data = data.is_none();
        let (kind2, icon) = if UnitAssetMenuData::get().is_shop_combat { ("MID_TUT_CATEGORY_TITLE_Battle", CustomMenuIcon::Weapon) } else { ("MID_SAVEDATA_SEQ_HUB", CustomMenuIcon::Day) };
        if let Some(data) = data.or(Some(&preview.preview_data)) {
            match page {
                EquipmentBoxPage::Flags => { Self::set_profile_flags(equipment, data.flag); }
                EquipmentBoxPage::Assets => {
                    let ubody = db.try_get_asset(AssetType::Body, data.ubody)
                        .or_else(|| db.try_get_asset(AssetType::Body, preview.original_assets[0]).filter(|_| no_data))
                        .map(|v| v.split_once("_").unwrap().1)
                        .unwrap_or("------".into());

                    let obody = "-------";
                    /*
                    db.hashes.get_obody(data.ubody).map(|v| v.to_string().as_str())
                        .or_else(|| db.hashes.o_body.get(&preview.original_assets[3]).filter(|_| no_data).map(|v| v.split_once("_").unwrap().1))
                        .unwrap_or("------".into());
                     */

                    set_content_data_slot(equipment, 1, CustomMenuIcon::Clothes.get_icon(), Some(format!("{} / {}", ubody, obody).into()));

                    let head = db.try_get_asset(AssetType::Head, data.uhead)
                        .or_else(|| db.try_get_asset(AssetType::Head, preview.original_assets[1]).filter(|_| no_data))
                        .map(|v| v.as_str().into())
                        .or_else(|| Some("------".into()));

                    set_content_data_slot(equipment, 2, CustomMenuIcon::Head.get_icon(), head);

                    let hair = db.try_get_asset(AssetType::Hair, data.uhair)
                        .or_else(|| db.try_get_asset(AssetType::Hair, preview.original_assets[2]).filter(|_| no_data))
                        .map(|v| v.as_str().into())
                        .or_else(|| Some("------".into()));

                    set_content_data_slot(equipment, 3, CustomMenuIcon::Hair.get_icon(), hair);

                    let rig = db.try_get_asset(AssetType::Rig, data.rig)
                        .or_else(|| db.try_get_asset(AssetType::Rig, preview.original_assets[15]).filter(|_| no_data))
                        .map(|v| v.as_str().into())
                        .or_else(|| Some("------".into()));

                    set_content_data_slot(equipment, 4, CustomMenuIcon::Body.get_icon(), rig);

                    let ohead = db.hashes.get_ohair(data.uhair)
                        .or_else(|| db.hashes.o_hair.get(&preview.original_assets[4]).map(|v| v.as_str().into()).filter(|_| no_data))
                        .or_else(|| db.hashes.get_ohair(preview.original_assets[2]).filter(|_| no_data))
                        .or_else(|| db.hashes.get_ohair(preview.original_assets[1]).filter(|_| no_data))
                        .or_else(|| Some("------".into()));

                    set_content_data_slot(equipment, 5, CustomMenuIcon::Head.get_icon(), ohead);
                }
                EquipmentBoxPage::AccessoryAssets => {
                    for x in 0..5 {
                        let name = db.try_get_asset(AssetType::Acc(x as u8), data.acc[x])
                            .or_else(|| db.try_get_asset(AssetType::Acc(x as u8), preview.original_assets[5 + x]).filter(|_| no_data))
                            .map(|v| v.as_str().into())
                            .or_else(|| Some("------".into()));
                        let icon =
                            if x < 3 { CustomMenuIcon::AccFace.get_icon() }
                            else if x == 3 { CustomMenuIcon::EngageCommon.get_icon() }
                            else { CustomMenuIcon::Shield.get_icon() };
                        set_content_data_slot(equipment, 1 + x, icon, name);
                    }
                }
                EquipmentBoxPage::RideMounts => {
                    for x in 0..5 {
                        set_content_data_slot(
                            equipment, 1 + x,
                            CustomMenuIcon::Mount(x as u8).get_icon(),
                            db.try_get_asset(AssetType::Mount(x as u8), data.mount[x]).map(|v| v.as_str().into())
                                .or_else(|| Some("------".into()))
                        );
                    }
                }
                EquipmentBoxPage::AOCAnimations => {
                    let gender = db.get_dress_gender_hash(data.ubody).unwrap_or(
                        if UnitAssetMenuData::get_current_dress_gender() == 2 {
                            engage_il2cpp::app::Gender::female() } else { engage_il2cpp::app::Gender::male() }
                    );
                    for x in 0..4 {
                        let hash = if gender == engage_il2cpp::app::Gender::female() { data.aoc_alt[x] } else { data.aoc[x] };
                        let aoc_name =
                            db.try_get_asset(AssetType::AOC(x as u8), hash)
                            .or_else(|| db.try_get_asset(AssetType::AOC(x as u8), preview.original_assets[10 + x]).filter(|_| no_data))
                            .map(|v| v.as_str().into()).or_else(|| Some("------".into()));

                        set_content_data_slot(equipment, 1 + x, CustomMenuIcon::SolaTail.get_icon(), aoc_name);
                    }
                    let voice = db.try_get_asset(AssetType::Voice, data.voice)
                        .or_else(|| db.try_get_asset(AssetType::Voice, preview.original_assets[14]).filter(|_| no_data))
                        .map(|v| v.as_str().into()).or_else(|| Some("------".into()));

                    set_content_data_slot(equipment, 5, CustomMenuIcon::TalkStory.get_icon(), voice);
                }
                EquipmentBoxPage::Color(kind) => {
                    let k = kind % 16;
                    let asset_colors = k < 8;
                    if asset_colors { set_content_data_slot(equipment, 1, icon.get_icon(), Some(format!("Viewing: {}", engage_il2cpp::app::Mess::get(kind2)).into())); }
                    let offset = if k >= 8 { 8 } else if k >= 4 { 4 } else { 0 } as usize;
                    let len = if asset_colors { 4 } else { 6 };
                    for x in 0..len  {
                        let color = x + offset;
                        let slot = if asset_colors { x + 2 } else { x };
                        let color_str =
                            if data.colors[color].has_color() && (!no_data || data.colors[color].values[3] != 0) {
                                format!("{}: {}", MenuText::get_command(1140 + color as i32), data.colors[color])
                            }
                            else if no_data {
                                format!("{}: {}/{}/{}",
                                    MenuText::get_command(1140 + color as i32),
                                    preview.original_color[4 * color],
                                    preview.original_color[4 * color + 1],
                                    preview.original_color[4 * color + 2],
                                )
                            }
                            else { format!("{}: --/--/--", MenuText::get_command(1140 + color as i32)) }.into();

                        set_content_data_slot(equipment, slot, CustomMenuIcon::Color.get_icon(), Some(color_str));
                        set_content_icon_color(equipment, slot, color as i32);
                    }
                }
                EquipmentBoxPage::Scaling(set) => {
                    set_content_data_slot(equipment, 1, icon.get_icon(), Some(format!("Viewing: {}", engage_il2cpp::app::Mess::get(kind2)).into()));
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
    pub fn change_equipment_box(self, equipment: engage_il2cpp::app::AccessoryEquipmentInfo) {
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
            Self::ProfilePreview(profile) => { Self::set_profile(equipment, Some(profile)); }
            Self::CurrentProfile => { Self::set_profile(equipment, None); }
            Self::CurrentProfilePage(page) => { Self::set_data(equipment, page, None); }
            Self::LoadData(page) => {
                let menu = UnitAssetMenuData::get();
                if let Some(data) = menu.loaded_data.get_selected_data() {
                    let name = data.get_filename().into();
                    if let Some(con) = get_content(equipment, 0) {
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
    pub fn change_cursor(equipment: engage_il2cpp::app::AccessoryEquipmentInfo, kind: Option<i32>) {
        if UnitAssetMenuData::is_photo_graph()  { return; }
        if let Some(k) = kind { equipment.show_cursor_3(AccessoryData_Kinds{value: k}); }
        else { equipment.hide_cursor(); }
    }
    pub fn set_cursor(kind: Option<i32>){
        if UnitAssetMenuData::is_photo_graph() { return; }
        if let Some(equipment) = get_equipment_box() { Self::change_cursor(equipment, kind); }
    }
}
pub fn get_equipment_box() -> Option<engage_il2cpp::app::AccessoryEquipmentInfo> {
    let go =  GameObject::find("EquipmentAcc");
    if !go.is_null() {
        let equip = go.get_component::<engage_il2cpp::app::AccessoryEquipmentInfo>();
        if !equip.is_null() { Some(equip) } else { None }
    }
    else { None }
}
pub fn get_content(equipment: engage_il2cpp::app::AccessoryEquipmentInfo, slot: i32) -> Option<engage_il2cpp::app::AccessoryMenuItemContent> {
    let item = equipment.m_menu_item_list().get(slot);
    if !item.is_null() {
        if let Some(content) = item.get_menu_item_content().try_cast::<engage_il2cpp::app::AccessoryMenuItemContent>() {
            return if !content.is_null() {Some(content) } else { None }
        }
    }
    None
}
pub fn set_content_data_slot(equipment: engage_il2cpp::app::AccessoryEquipmentInfo, slot: usize, icon: Option<engage_il2cpp::unity_engine::Sprite>, name: Option<unity2::Il2CppString>) {
    let menu_item = equipment.m_menu_item_list().get(slot as i32);
    if !menu_item.is_null() {
        if let Some(content) = menu_item.get_menu_item_content().try_cast::<engage_il2cpp::app::AccessoryMenuItemContent>() {
            set_icon_text_to_content(content, icon, name);
        }
    }
}
pub fn set_icon_text_to_content(content: engage_il2cpp::app::AccessoryMenuItemContent, icon: Option<engage_il2cpp::unity_engine::Sprite>, name: Option<unity2::Il2CppString>) {
    if let Some(icon) = icon {
        content.m_kind_icon_image().set_color(engage_il2cpp::unity_engine::Color{r: 1.0, g: 1.0, b: 1.0, a: 1.0});
        content.m_kind_icon_object().set_active(true);
        content.m_kind_icon_image().set_sprite(icon);
    }
    else { content.m_kind_icon_object().set_active(false); }
    if let Some(name) = name { content.m_name_text().set_m_text(name); }
}
pub fn set_content_icon_color(equipment: engage_il2cpp::app::AccessoryEquipmentInfo, slot: usize, kind: i32) {
    let item = equipment.m_menu_item_list().get(slot as i32);
    if !item.is_null() {
        if let Some(content) = item.get_menu_item_content().try_cast::<engage_il2cpp::app::AccessoryMenuItemContent>() {
            content.m_kind_icon_object().set_active(true);
            set_icon_to_color(content, kind);
        }
    }
}
pub fn set_icon_to_color(content: engage_il2cpp::app::AccessoryMenuItemContent, kind: i32) {
    let preview = UnitAssetMenuData::get_preview();
    if preview.preview_data.colors[kind as usize].has_color() {
        content.m_kind_icon_object().set_active(true);
        content.m_kind_icon_image().set_sprite(engage_il2cpp::unity_engine::Sprite::null());
        let c = UnitAssetMenuData::get_preview().preview_data.colors[kind as usize].get_f32();
        content.m_kind_icon_image().set_color(engage_il2cpp::unity_engine::Color{r: c[0], g: c[1], b: c[2], a: 1.0});
    }
    else {
        let i = 4*kind as usize;
        if preview.original_color[i] > 0 || preview.original_color[i+1] > 0 || preview.original_color[i+2] > 0 {
            content.m_kind_icon_image().set_sprite(engage_il2cpp::unity_engine::Sprite::null());
            content.m_kind_icon_object().set_active(true);
            content.m_kind_icon_image().set_color(
                engage_il2cpp::unity_engine::Color{
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
pub fn set_detail_box(name: Option<unity2::Il2CppString>, help: Option<unity2::Il2CppString>, body: Option<unity2::Il2CppString>, sprite: Option<engage_il2cpp::unity_engine::Sprite>) {
    if UnitAssetMenuData::is_photo_graph()  { return; }
    let helpwdw = GameObject::find("WdwAccHelp");
    if !helpwdw.is_null() {
        let detail = helpwdw.get_component::<engage_il2cpp::app::AccessoryDetailInfoWindow>();
        if !detail.is_null() {
            if let Some(help) = help { detail.m_message().set_m_text(help); }
            if let Some(name) = name { detail.m_accessory_name().set_m_text(name); }
            if let Some(body) = body { detail.m_body_parts().get(0).m_text().set_m_text(body); }
            if let Some(sprite) = sprite {
                detail.m_body_parts().get(0).m_image().set_sprite(sprite);
            }
        }
    }
}