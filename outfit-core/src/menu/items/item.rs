use std::fs;
use engage::{
    dialog::BasicDialog2, gamevariable::GameVariableManager,
    mess::Mess, pad::{NpadButton, Pad}, random::Random,
    sequence::{
        hubaccessory::{HubAccessoryShopSequence, room::HubAccessoryRoom}, photograph::*
    },
    spriteatlasmanager::FaceThumbnail, tmpro::TextMeshProUGUI
};
use unity::{prelude::Il2CppString, system::action::{Action, Action1}};
use crate::{
    is_up_down_press, left_right_enclose, r_l_press, AssetType, THUMB_DIR,
    data::{items::Profile, room::hub_room_set_by_result},
    menu::{icons::CustomMenuIcon, items::{AssetFlag, CustomAssetMenuKind, CustomMenuItem}, *},
    localize::{MenuText, MenuTextCommand}
};
pub use CustomAssetMenuItemKind::*;
pub use CustomAssetMenuKind::*;

pub const PROFILE_MID: [&str; 4] = ["MID_TUT_CATEGORY_TITLE_Battle", "MID_MENU_ENGAGE_COMMAND", "MID_SAVEDATA_SEQ_HUB", "MCID_M007"];

#[repr(C)]
#[derive(PartialEq, Copy, Clone)]
pub enum CustomAssetMenuItemKind {
    NoItem, //-1
    Asset(AssetType),   //0
    ProfileItem(Profile),    //1
    FlagMenuItem(AssetFlag),    //2
    Data(AssetDataMode),
    CurrentProfile, // 7
    UnitName,   //  9
    ResetColor(u8), //  60  + color
    ScaleMenuItem(u8),  // 70 + scale
    RGBA{kind: u8, color: u8},  // 100 + 4*kind + color
    Menu(CustomAssetMenuKind),  // 1000 + menu index
    OutfitDataFile,
    CurrentData,
    PresetAppearance,
    Pause,
    Item,
    UnitInventorySubMenuItem,
    FaceThumb,
}
impl CustomAssetMenuItemKind {
    pub fn can_facial(&self) -> bool {
        match self {
            UnitName|CurrentProfile|ScaleMenuItem(_)|RGBA{kind: _, color: _}|ProfileItem(_) => false,
            _ => true,
        }
    }
    pub fn to_index(&self) -> i32 {
        match self {
            UnitInventorySubMenuItem => 0,
            CurrentProfile => 1,
            OutfitDataFile => 2,
            UnitName => 4,
            Data(AssetDataMode::Export) => 5,
            Data(AssetDataMode::Import) => 6,
            Data(AssetDataMode::ExportPreview) => 9,
            CurrentData => 7,
            PresetAppearance => 8,
            ProfileItem(profile) => 10 + profile.to_index() as i32,
            FlagMenuItem(ty) => { 20 + ty.get_rel_index() }
            ResetColor(color) =>  30 + *color as i32,
            Asset(ty) => { 40 + ty.to_index() }
            RGBA { kind, color } => 100 + (*kind as i32) * 4 + (*color as i32),  //200 -> 232
            ScaleMenuItem(ty) => 150 + *ty as i32,  // 300 -> 316
            Menu(menu) => 1000 + menu.to_index(),
            Pause => -2,
            Item => -3,
            FaceThumb => -4,
            NoItem => -1,
        }
    }
    pub fn from_index(index: i32) -> Self {
        match index {
            0 => UnitInventorySubMenuItem,
            1 => CurrentProfile,
            2 => OutfitDataFile,
            4 => UnitName,
            5 => Data(AssetDataMode::Export),
            6 => Data(AssetDataMode::Import),
            7 => CurrentData,
            8 => PresetAppearance,
            9 => Data(AssetDataMode::ExportPreview),
            10..20 => ProfileItem(Profile::from_index(index-10)),
            20..30 => AssetFlag::from_rel_index(index - 20).map(|x| FlagMenuItem(x)).unwrap_or(NoItem),
            30..38 => ResetColor(index as u8 - 30),
            40..100 => AssetType::from_rel_index(index - 40).map(|x| Asset(x)).unwrap_or(NoItem),
            100..132 => {
                let offset = index - 100;
                let kind = (offset / 4) as u8;
                let color = (offset % 4) as u8;
                RGBA {kind, color}
            }
            150..166 => { ScaleMenuItem(index as u8 - 150) }
            -1 => Pause,
            -2 => Item,
            -4 => FaceThumb,
            _ => NoItem,
        }
    }
    pub fn on_select(&self, menu_item: &CustomAssetMenuItem) {
        if *self == UnitInventorySubMenuItem { return; }
        let help = self.get_help(menu_item);
        let body = menu_item.menu.menu_kind.get_body(menu_item);
        let icon = self.get_icon(menu_item);
        let name = self.get_detail_box_name(menu_item);
        set_detail_box(name, Some(help), Some(body), icon.get_icon());
        self.get_equipment_box_type(menu_item).update();
        match self {
            CurrentData => {
                let data = UnitAssetMenuData::get();
                data.loaded_data.selected_index = None;
                let box_state = data.loaded_data.equipment_box_state;
                EquipmentBoxMode::CurrentProfilePage(box_state).update();
                let pressed = is_up_down_press();
                if pressed { data.is_changed = true; }
                else {
                    hub_room_set_by_result(None, ReloadType::ForcedUpdate);
                    data.is_changed = false;
                }
                set_detail_box(name, Some(help), Some(body), icon.get_icon());
            }
            OutfitDataFile => {
                let data = UnitAssetMenuData::get();
                let current = if menu_item.index == 0 { None } else { Some(menu_item.index-1) };
                if data.loaded_data.selected_index != current {
                    data.loaded_data.selected_index = current;
                    let pressed = is_up_down_press();
                    if pressed { data.is_changed = true; }
                    else {
                        let result = UnitAssetMenuData::get_result();
                        let menu = UnitAssetMenuData::get();
                        if let Some(loaded) = menu.loaded_data.selected_index.and_then(|i| menu.loaded_data.loaded_data.get_mut(i as usize)) {
                            let flag = loaded.data.flag;
                            loaded.data.flag |= 193;
                            loaded.data.set_result(result, 2, false, false);
                            loaded.data.flag = flag;
                        }
                        hub_room_set_by_result(Some(result), ReloadType::ForcedUpdate);
                        data.is_changed = false;
                    }
                    let box_state = data.loaded_data.equipment_box_state;
                    EquipmentBoxMode::LoadData(box_state).update();
                }
                data.loaded_data.selected_index = current;
                let name = self.get_detail_box_name(menu_item);
                set_detail_box(name, Some(help), Some(body), Profile::from_index(data.loaded_data.profile).get_icon().get_icon());
            }
            Asset(ty) => {
                let pressed = is_up_down_press();
                if pressed { UnitAssetMenuData::get().is_changed = true; }
                ty.update_preview(menu_item, true, !pressed);
            }
            RGBA {kind, color: _} => {
                let menu_data = UnitAssetMenuData::get_preview();
                let color_kind = *kind as usize;
                let result = UnitAssetMenuData::get_result();
                result.unity_colors[color_kind].r = menu_data.color_preview[4*color_kind] as f32 / 255.0;
                result.unity_colors[color_kind].g = menu_data.color_preview[4*color_kind+1] as f32 / 255.0;
                result.unity_colors[color_kind].b = menu_data.color_preview[4*color_kind+2] as f32 / 255.0;
                let cursor_pos = if color_kind < 4 { color_kind + 2 } else { color_kind - 2 };
                EquipmentBoxMode::set_cursor(Some(cursor_pos as i32));
                hub_room_set_by_result(Some(result), ReloadType::ColorScale);
            }
            Menu(menu) => {
                match menu {
                    ShopBody(_) => EquipmentBoxMode::set_cursor(Some(1)),
                    Head => EquipmentBoxMode::set_cursor(Some(2)),
                    Hair => EquipmentBoxMode::set_cursor(Some(3)),
                    Rig => EquipmentBoxMode::set_cursor(Some(4)),
                    VoiceSelection => EquipmentBoxMode::set_cursor(Some(5)),
                    ColorSelection(kind) => EquipmentBoxMode::set_cursor(Some(if *kind < 4 { 2 + *kind } else { *kind - 2 } as i32)),
                    RGBAMenu(kind) => {
                        let color_kind = *kind as usize;
                        let menu_data = UnitAssetMenuData::get_preview();
                        let result = UnitAssetMenuData::get_result();
                        let has_color = menu_data.color_preview[4*color_kind] as i32 + menu_data.color_preview[4*color_kind+1]as i32+ menu_data.color_preview[4*color_kind+2] as i32;
                        if has_color > 0{
                            result.unity_colors[color_kind].r = menu_data.color_preview[4*color_kind] as f32 / 255.0;
                            result.unity_colors[color_kind].g = menu_data.color_preview[4*color_kind+1] as f32 / 255.0;
                            result.unity_colors[color_kind].b = menu_data.color_preview[4*color_kind+2] as f32 / 255.0;
                        }
                        hub_room_set_by_result(Some(result), ReloadType::ColorScale);
                    }
                    _ => EquipmentBoxMode::set_cursor(None),
                }
            }
            ResetColor(color_kind) => {
                let color_kind = *color_kind as usize;
                let menu_data = UnitAssetMenuData::get_preview();
                let result = UnitAssetMenuData::get_result();
                result.unity_colors[color_kind].r = menu_data.original_color[4*color_kind] as f32 / 255.0;
                result.unity_colors[color_kind].g = menu_data.original_color[4*color_kind+1] as f32 / 255.0;
                result.unity_colors[color_kind].b = menu_data.original_color[4*color_kind+2] as f32 / 255.0;
                let cursor_pos = if color_kind < 4 { color_kind + 2 } else { color_kind - 2 };
                EquipmentBoxMode::set_cursor(Some(cursor_pos as i32));
                hub_room_set_by_result(Some(result), ReloadType::ColorScale);
            }
            ScaleMenuItem(kind) => {
                let menu_data = UnitAssetMenuData::get_preview();
                for x in 0..16 {
                    if menu_data.scale_preview[x] == 0 { menu_data.scale_preview[x] = menu_data.original_scaling[x]; }
                }
                let cursor_index = (*kind % 4) + 2;
                EquipmentBoxMode::set_cursor(Some(cursor_index as i32));
                if *kind != 0 { hub_room_set_by_result(None, ReloadType::Scale); }
            }
            PresetAppearance => {
                EquipmentBoxMode::set_cursor(None);
                if UnitAssetMenuData::is_photo_graph(){
                    let db = get_outfit_data();
                    if let Some(appearance) = db.dress.personal.get(menu_item.hash as usize) {
                        UnitAssetMenuData::get().preview.preview_data.set_from_preset(appearance);
                    }
                }
                else {
                    let data = UnitAssetMenuData::get();
                    let box_state = data.loaded_data.equipment_box_state;
                    if box_state == EquipmentBoxPage::Flags {
                        data.loaded_data.equipment_box_state = EquipmentBoxPage::Assets;
                    }
                    EquipmentBoxMode::LoadData(data.loaded_data.equipment_box_state).set_preset_appearance(menu_item.hash);
                }
                UnitAssetMenuData::get().is_changed = true;
            }
            Pause => {
                if let Some(dispos) = PhotographTopSequence::get_photograph_sequence().map(|p| &mut p.dispos_manager) {
                    if let Some(pause) = dispos.current_dispos_info.pause_data_list.get(menu_item.index as usize){
                        dispos.current_dispos_info.current_pause_data = Some(pause);
                        dispos.current_dispos_info.set_up_pause();
                    }
                }
            }
            Item  => {
                if let Some(dispos) = PhotographTopSequence::get_photograph_sequence().map(|p| &mut p.dispos_manager) {
                    if let Some(pause) = dispos.current_dispos_info.weapon_data_list.get(menu_item.index as usize){
                        dispos.current_dispos_info.weapon_data = Some(pause);
                        dispos.current_dispos_info.setup_weapon();
                    }
                }
            }
            _ => { EquipmentBoxMode::set_cursor(None); }
        }
    }
    pub fn build_attribute(&self) -> BasicMenuItemAttribute {
        let emblem = UnitAssetMenuData::get_unit().is_none();
        let dvc = UnitAssetMenuData::get().is_dvc;
        let photo = UnitAssetMenuData::is_photo_graph();
        match self {
            Menu(menu) => menu.build_attribute(emblem),
            Data(_) => { if photo { BasicMenuItemAttribute::Hide } else { BasicMenuItemAttribute::Enable } }
            FlagMenuItem(AssetFlag::EnableCrossDressing) => if emblem { BasicMenuItemAttribute::Hide } else { BasicMenuItemAttribute::Enable },
            FlagMenuItem(AssetFlag::EngagedAnimation)|FlagMenuItem(AssetFlag::EngageOutfit) => if emblem || photo { BasicMenuItemAttribute::Hide } else { BasicMenuItemAttribute::Enable },
            FlagMenuItem(AssetFlag::UseFaceThumbnail) => {
                if !emblem && UnitAssetMenuData::is_unit_info() { BasicMenuItemAttribute::Enable } else { BasicMenuItemAttribute::Hide }
            }
            FlagMenuItem(AssetFlag::RandomAppearance) => if emblem || !dvc || photo { BasicMenuItemAttribute::Hide } else { BasicMenuItemAttribute::Enable },
            UnitName => if emblem { BasicMenuItemAttribute::Hide } else { BasicMenuItemAttribute::Enable },
            _ => BasicMenuItemAttribute::Enable,
        }
    }
}

impl CustomMenuItem for CustomAssetMenuItemKind {
    fn get_icon(&self, menu_item: &CustomAssetMenuItem) -> CustomMenuIcon {
        match self {
            RGBA {kind: _, color: _} => CustomMenuIcon::Star,
            ResetColor(_) => CustomMenuIcon::Star,
            ScaleMenuItem(_) => CustomMenuIcon::StarBlank,
            Asset(ty) => ty.get_icon(menu_item),
            CurrentProfile => CustomMenuIcon::KeyItem,
            ProfileItem(profile) => { profile.get_icon() }
            FlagMenuItem(flag) => flag.get_icon(menu_item),
            Menu(menu) => menu.get_icon(menu_item),
            UnitName => CustomMenuIcon::SilverCard,
            OutfitDataFile|Data(_) => CustomMenuIcon::Satchel,
            CurrentData => { Profile::from_index(UnitAssetMenuData::get_preview().selected_profile).get_icon() }
            PresetAppearance => CustomMenuIcon::Body,
            _ => CustomMenuIcon::NoIcon,
        }
    }
    fn get_equipment_box_type(&self, menu_item: &CustomAssetMenuItem) -> EquipmentBoxMode {
        match self {
            Asset(asset) => asset.get_equipment_box_type(menu_item),
            FlagMenuItem(flag) => flag.get_equipment_box_type(menu_item),
            RGBA {kind, color: _} => EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Color(*kind)),
            ResetColor(kind) => EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Color(*kind)),
            ScaleMenuItem(kind) => EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Scaling(*kind/4)),
            ProfileItem(profile) => EquipmentBoxMode::ProfilePreview(*profile),
            Menu(menu) => menu.get_equipment_box_type(menu_item),
            _ => EquipmentBoxMode::CurrentProfile,
        }
    }
    fn get_name(&self, menuitem: &CustomAssetMenuItem) -> &'static Il2CppString {
        let idx = self.to_index();
        match self {
            UnitInventorySubMenuItem => { MenuTextCommand::Outfits.get() }
            PresetAppearance|Pause|Item => { menuitem.name }
            OutfitDataFile => { menuitem.name }
            Menu(menu) => menu.get_menu_item_name().unwrap_or_else(||menuitem.name.to_string().into()),
            Asset(ty) => ty.get_name(menuitem),
            FlagMenuItem(flag) => flag.get_name(menuitem),
            ScaleMenuItem(scale_index) => {
                let i = *scale_index as i32;
                let v = UnitAssetMenuData::get_preview().scale_preview[i as usize];
                format!("{}: {}", MenuText::get_command(idx), v as f32 / 100.0).into()
            }
            RGBA{kind, color} => {
                let kind = *kind as usize;
                let r = *color as usize;
                let preview = UnitAssetMenuData::get_preview();
                let v = preview.color_preview[4*kind + r];
                format!("{}: {}", MenuText::get_command(90 + *color as i32), v).into()
            }
            CurrentProfile => {
                let emblem = UnitAssetMenuData::get().god_mode;
                let preview = UnitAssetMenuData::get_preview();
                format!("{}: {}", MenuText::get_command(1), get_profile_name(preview.selected_profile, emblem)).into()
            }
            ProfileItem(p) => {
                let emblem = UnitAssetMenuData::get().god_mode;
                let i = if menuitem.index == 1 && emblem { 3 } else { menuitem.index } as usize;
                format!("{}: {}", Mess::get(PROFILE_MID[i]), p.get_name()).into()
            }
            UnitName => {
                if let Some(unit) = UnitAssetMenuData::get_shop_unit().filter(|v| v.edit.name.is_some()){
                    format!("{} [{}]", unit.get_name(), MenuTextCommand::on_off(unit.edit.is_enabled())).into()
                }
                else { "Custom Name".into() }
            }
            Data(data) => { data.get_name(menuitem) }
            NoItem => { Mess::get_item_none2() }
            _ => { MenuText::get_command(idx) }
        }
    }
    fn get_detail_box_name(&self, menuitem: &CustomAssetMenuItem) -> Option<&'static Il2CppString> {
        let idx = self.to_index();
        match self {
            OutfitDataFile => {
                if let Some(select) = UnitAssetMenuData::get().loaded_data.get_selected_data() { select.path.file_stem().and_then(|x| x.to_str()).map(|x| x.into()) }
                else { None }
            }
            Asset(ty) => ty.get_detail_box_name(menuitem),
            FlagMenuItem(flag) => flag.get_detail_box_name(menuitem),
            RGBA {kind: _, color} => Some(MenuText::get_command(90 + *color as i32)),
            ProfileItem(profile) => { Some(profile.get_name()) }
            Menu(menu) => { menu.get_detail_box_name(menuitem) }
            CurrentProfile => { Some(get_current_profile_name()) }
            Data(item) => { item.get_detail_box_name(menuitem) }
            UnitName => { Some("Unit Name".into()) }
            PresetAppearance => { Some(menuitem.name) }
            FaceThumb => { Some(menuitem.name.to_string().trim_end_matches(".png").into()) }
            _ => { Some(MenuText::get_command(idx)) }
        }
    }
    fn get_help(&self, menuitem: &CustomAssetMenuItem) -> &'static Il2CppString {
        let idx = self.to_index();
        match self {
            ResetColor(kind) => {
                let set = UnitAssetMenuData::get_set_color_str(*kind as i32);
                let original = UnitAssetMenuData::get_original_color_str(*kind as i32);
                if set != original {
                    format!("{} {} {}\n{}", MenuText::get_help(30).unwrap(),
                        MenuTextCommand::Reset.insert_right(set),
                        MenuTextCommand::Original.insert_right(original),
                        MenuTextCommand::A.to_right(MenuTextCommand::Confirm),
                    ).into()
                }
                else { MenuText::get_help(31).unwrap() }
            }
            Asset(ty) => ty.get_help(menuitem),
            FlagMenuItem(flag) => flag.get_help(menuitem),
            RGBA {kind, color} => {
                let k = *kind as i32;
                let c = *color as i32;
                let current = UnitAssetMenuData::get_current_color(*kind as i32, *color as i32);
                let original = UnitAssetMenuData::get().preview.color_preview[(4*k + c) as usize];
                let enabled = if UnitAssetMenuData::get_flag() & 1 == 0 { " <color=\"yellow\">[Not Active]</color>" } else { ""};
                if current != original {
                    format!("{} {}{}\n{} {} {}",
                        MenuTextCommand::LeftRight, MenuTextCommand::Original.insert_right(original), enabled,
                        MenuTextCommand::A.to_right(MenuTextCommand::Confirm),
                        MenuTextCommand::X.to_right(MenuTextCommand::Random),
                        MenuTextCommand::Minus.to_right(MenuTextCommand::Reset),
                    ).into()
                }
                else {
                    format!("{}{}{}\n{}",
                        MenuTextCommand::LeftRight, MenuTextCommand::Original.insert_right(original), enabled,
                        MenuTextCommand::X.to_right(MenuTextCommand::Random)
                    ).into()
                }
            },
            ScaleMenuItem(scale_index) => {
                let i = *scale_index as usize;
                let preview = UnitAssetMenuData::get_preview();
                let original = preview.original_scaling[i] as f32 / 100.0;
                let current = preview.scale_preview[i] as f32 / 100.0;
                let arrow =
                    if preview.scale_preview[i] == 1 { MenuTextCommand::Right.get() }
                    else if preview.scale_preview[i] == 1000 { MenuTextCommand::Left.get() }
                    else { MenuTextCommand::LeftRight.get() };

                let enabled = if UnitAssetMenuData::get_flag() & 64 == 0 { " <color=\"yellow\">[Not Active]</color>" } else { ""};
                if original != current {
                    format!("{} {}{}\n{} {} {}",
                        arrow, MenuTextCommand::Original.insert_right(original), enabled,
                        MenuTextCommand::A.to_right(MenuTextCommand::Confirm),
                        MenuTextCommand::X.to_right(MenuTextCommand::Random),
                        MenuTextCommand::Minus.to_right(MenuTextCommand::Reset),
                    ).into()
                }
                else {
                    format!("{} {}{}.\n{}",
                        arrow, MenuTextCommand::Original.insert_right(original), enabled,
                        MenuTextCommand::X.to_right(MenuTextCommand::Random)
                    ).into()
                }
            },
            ProfileItem(_) => {
                let emblem = UnitAssetMenuData::get().god_mode;
                let i = if menuitem.index == 1 && emblem { 3 } else { menuitem.index };
                MenuText::get_help(10 + i).unwrap()
            },
            Menu(menu) => menu.get_help(menuitem),
            UnitName => {
                let help = MenuText::get_help(400).unwrap();
                if let Some(unit) = UnitAssetMenuData::get_shop_unit() {
                    if let Some(name) = unit.edit.name {
                        let help_idx = if unit.person.flag.value & 128 == 0 && unit.person.parent.index > 1 { 401 } else { 400 };
                        format!("{}\nSet name: {}", MenuText::get_help(help_idx).unwrap(), name).into()
                    } else { format!("{}\nDefault: {}", help, Mess::get_name(unit.person.pid)).into() }
                }
                else { help }
            }
            CurrentProfile => { get_current_profile_assignment_text().into() }
            _ => { MenuText::get_help(idx).unwrap_or(format!("MenuItemHelp #{}", idx).into()) }
        }
    }
    fn get_body(&self, menuitem:  &CustomAssetMenuItem) -> &'static Il2CppString {
        match self {
            CurrentData => { get_current_profile_name() }
            Asset(ty) => ty.get_body(menuitem),
            FlagMenuItem(flag) => flag.get_body(menuitem),
            ResetColor(kind) => MenuText::get_command(1140+ *kind as i32),
            RGBA {kind, color: _} => { MenuText::get_command(1140 + *kind as i32) },
            Data(_) => MenuTextCommand::Data.get(),
            CurrentProfile|ProfileItem(_) => MenuText::get_command(1),
            OutfitDataFile => {
                if menuitem.index == 0 { "".into() }
                else {
                    let emblem = UnitAssetMenuData::get().god_mode;
                    let saved_profile = UnitAssetMenuData::get().loaded_data.profile;
                    left_right_enclose(&format!("{}: {}", MenuTextCommand::Copy, get_profile_name(saved_profile, emblem)))
                }
            }
            PresetAppearance => {
                if menuitem.padding == 1 { MenuTextCommand::Emblem.get() }
                else { MenuTextCommand::Personal.get() }
            }
            _ => "".into(),
        }
    }
    fn a_call(&self, menuitem: &mut CustomAssetMenuItem) -> BasicMenuResult {
        match self {
            UnitInventorySubMenuItem => { 
                if let Some(parent) = menuitem.menu.get_parent() {
                    CustomAssetMenu::create_unit_info_bind(parent, SortieSelectionUnitManager::get_unit());
                }
                BasicMenuResult::close_decide()
            }
            Asset(ty) => ty.a_call(menuitem),
            ResetColor(kind) => {
                let i = *kind as usize;
                let preview = UnitAssetMenuData::get_preview();
                for x in 0..4 {
                    if preview.preview_data.colors[i].values[x] != 0 {
                        preview.preview_data.colors[i].values[x] = 0;
                        preview.color_preview[4 * i + x] = preview.original_color[4 * i + x];
                    }
                }
                UnitAssetMenuData::reload_unit(ReloadPreview::NoScaleColor, true, None);
                BasicMenuResult::se_decide()
            }
            RGBA{kind, color} => {
                let (i, c) = (*kind as usize, *color as usize);
                let preview = UnitAssetMenuData::get_preview();
                if preview.original_color[4 * i + c] != preview.color_preview[4 * i + c] {
                    preview.preview_data.colors[i].values[c] = preview.color_preview[4 * i + c];
                    self.on_select(menuitem);
                    BasicMenuResult::se_decide()
                }
                else { BasicMenuResult::se_miss() }
            }
            ScaleMenuItem(scale_index) => {
                let i = *scale_index as usize;
                let preview = UnitAssetMenuData::get_preview();
                if preview.original_scaling[i] != preview.scale_preview[i] {
                    preview.preview_data.scale[i] = preview.scale_preview[i];
                    self.on_select(menuitem);
                    BasicMenuResult::se_decide()
                }
                else { BasicMenuResult::se_miss() }
            }
            Data(data) => data.a_call(menuitem),
            Menu(menu) => {
                if *menu == MainShop { menuitem.menu.kind = 0; } else { menuitem.menu.kind = 1; }
                menuitem.menu.save_current_select();
                menuitem.menu.full_menu_item_list.clear();
                menu.create_menu_items(menuitem.menu);
                menuitem.menu.menu_kind = *menu;
                menuitem.menu.rebuild_menu();
                BasicMenuResult::se_cursor()
            }
            UnitName => {
                if let Some(unit) = UnitAssetMenuData::get_shop_unit() {
                    UnitAssetMenuData::get().name_set = false;
                    let initial = unit.edit.name.or(Some(Mess::get_name(unit.person.pid)));
                    let header = Some(Mess::get("MID_GAMESTART_PLAYER_NAME_INPUT").to_string().into());
                    let sub_text = Some( "".into());
                    let limit = 20;
                    let action = Action1::new_with_method(Some(unit), set_unit_name);
                    engage::keyboard::SoftwareKeyboard::create_bind(menuitem.menu, limit, initial, header, sub_text, 0, Some(action));
                }
                BasicMenuResult::se_cursor()
            }
            OutfitDataFile => {
                let data = UnitAssetMenuData::get();
                let result =
                if menuitem.index > 0 {
                    let preview = UnitAssetMenuData::get_preview();
                    let db = get_outfit_data();
                    if let Some(new_data) = data.loaded_data.loaded_data.get( menuitem.index as usize - 1){
                        let selected_profile = data.loaded_data.profile as usize;
                        let hash = data.preview.person;
                        if let Some(profile) = data.data.iter_mut().find(|x| x.person == hash )
                            .and_then(|unit_data| unit_data.profile.get_mut(selected_profile))
                        {
                            let current_dress = profile.ubody;
                            let current_dress_gender = db.get_dress_gender_hash(current_dress)
                                .or_else(||db.get_dress_gender_hash(data.preview.original_assets[0]))
                                .unwrap_or(Gender::None);
                            let flag = profile.flag;
                            *profile = new_data.data.clone();
                            if flag & 128 == 0 && db.try_get_asset(AssetType::Body, new_data.data.ubody).is_some() {
                                if db.get_dress_gender_hash(new_data.data.ubody).is_some_and(|v| v != current_dress_gender) {
                                    profile.ubody = current_dress;
                                }
                                profile.flag &= !128;
                            }
                            if selected_profile == preview.selected_profile as usize { UnitAssetMenuData::set_preview(profile); }
                        }
                    }
                    BasicMenuResult::se_decide()
                }
                else { BasicMenuResult::se_miss() };
                CustomAssetMenu::b_call(menuitem.menu, None);
                result
            }
            FaceThumb => {
                if let Some(unit) = UnitAssetMenuData::get_unit() {
                    if let Some(keys) = crate::capture::get_unit_face_keys(unit){
                        if let Some(sprite) = FaceThumbnail::get_item(format!("LOAD_{}", menuitem.hash)){
                            FaceThumbnail::try_insert(keys.2, sprite);
                            UnitAssetMenuData::get().loaded_data.selected_index = Some(menuitem.hash);
                            let key = format!("G_Face_{}", keys.0);
                            let name = menuitem.name.to_string();
                            if !GameVariableManager::exist(key.as_str()) { GameVariableManager::make_entry_str(key.as_str(), name); }
                            else { GameVariableManager::set_string(key.as_str(), name); }
                            FaceThumbnail::try_insert(keys.0, sprite);
                        }
                    }
                }
                CustomAssetMenu::b_call(menuitem.menu, None);
                BasicMenuResult::se_decide()
            }
            PresetAppearance => {
                let db = get_outfit_data();
                let preview = UnitAssetMenuData::get_preview();
                if let Some(appearance) = db.dress.personal.get(menuitem.hash as usize) {
                    preview.preview_data.set_from_preset(appearance);
                    BasicMenuResult::se_decide()
                }
                else { BasicMenuResult::se_miss() }
            }
            FlagMenuItem(flag) => { flag.a_call(menuitem) }
            _ => { BasicMenuResult::new() }
        }
    }
    fn x_call(&self, menuitem: &mut CustomAssetMenuItem) -> BasicMenuResult {
        match self {
            ScaleMenuItem(scale_index) => {
                let i = *scale_index as usize;
                let preview = UnitAssetMenuData::get_preview();
                let random_value = get_random_scaling(i as i32, Random::get_game());
                if random_value > 0  {
                    preview.scale_preview[i] = random_value as u16;
                    preview.preview_data.scale[i] = random_value as u16;
                    menuitem.rebuild_text();
                    hub_room_set_by_result(None, ReloadType::Scale);
                    BasicMenuResult::se_decide()
                }
                else { BasicMenuResult::se_miss() }
            }
            RGBA{kind, color} => {
                let rng = Random::get_game();
                let (i, c) = (*kind as usize, *color as usize);
                let preview = UnitAssetMenuData::get_preview();
                let random_value = rng.get_value(255) as u8;
                if random_value != preview.original_color[4 * i + c] {
                    preview.color_preview[4 * i + c] = random_value;
                    preview.preview_data.colors[i].values[c] = random_value;
                    UnitAssetMenuData::reload_unit(ReloadPreview::Color(*kind as i32), true, None);
                    menuitem.rebuild_text();
                    BasicMenuResult::se_decide()
                }
                else { BasicMenuResult::se_miss() }
            }
            Asset(AssetType::Body) => {
                if !UnitAssetMenuData::get().god_mode && !UnitAssetMenuData::is_photo_graph() {
                    UnitAssetMenuData::get_preview().preview_data.break_body = menuitem.hash;
                    menuitem.menu.full_menu_item_list.iter_mut().for_each(|v|{v.rebuild_text(); });
                    BasicMenuResult::se_decide()
                }
                else { BasicMenuResult::new() }
            }
            _ => { BasicMenuResult::new() }
        }
    }
    fn minus_call(&self, menuitem: &mut CustomAssetMenuItem) -> BasicMenuResult {
        match self {
            ScaleMenuItem(scale_index) => {
                let i = *scale_index as usize;
                let preview = UnitAssetMenuData::get_preview();
                if preview.scale_preview[i] != preview.original_scaling[i] { preview.scale_preview[i] = preview.original_scaling[i]; }
                if UnitAssetMenuData::is_photo_graph()  { preview.preview_data.scale[i] = preview.scale_preview[i]; }
                preview.preview_data.scale[i] = 0;
                hub_room_set_by_result(None, ReloadType::Scale);
                menuitem.rebuild_text();
                BasicMenuResult::se_decide()
            }
            RGBA { kind, color} => {
                let (i, c) = (*kind as usize, *color as usize);
                let preview = UnitAssetMenuData::get_preview();
                if preview.original_color[4 * i + c] != preview.color_preview[4 * i + c] {
                    preview.color_preview[4 * i + c] = preview.original_color[4 * i + c];
                    preview.preview_data.colors[i].values[c] = 0;
                    UnitAssetMenuData::reload_unit(ReloadPreview::Color(*kind as i32), true, None);
                    menuitem.rebuild_text();
                    BasicMenuResult::se_decide()
                }
                else { BasicMenuResult::se_miss() }
            }
            PresetAppearance => {
                let preview = UnitAssetMenuData::get_preview();
                let flags = preview.preview_data.flag;
                preview.preview_data = PlayerOutfitData::new_with_flag(flags);
                UnitAssetMenuData::reload_unit(ReloadPreview::Forced, true, None);
                BasicMenuResult::se_decide()
            }
            Asset(AssetType::AOC(_)) => {
                if !UnitAssetMenuData::get().god_mode && UnitAssetMenuData::is_unit_info() {
                    let use_thumbnail = UnitAssetMenuData::get_person_flag() & 8 != 0;
                    crate::capture::capture_unit_info(menuitem.menu, true, use_thumbnail);
                    BasicMenuResult::se_cursor()
                } else { BasicMenuResult::se_miss() }
            }
            FaceThumb => {
                let message = format!("Delete '{}'?", menuitem.name);
                let action = Action::new_method_mut(Some(menuitem), delete_face_item);
                BasicDialog2::create_confirm_cancel_bind(menuitem.menu, message, Some(action));
                BasicMenuResult::se_cursor()
            }
            _ => { BasicMenuResult::new() }
        }
    }
    fn custom_call(&self, menuitem: &mut CustomAssetMenuItem) -> BasicMenuResult {
        let menu = UnitAssetMenuData::get();
        let changed = menu.is_changed;
        let is_up_down = is_up_down_press();
        let pad = get_instance::<Pad>();
        let photo = menu.mode == MenuMode::PhotoGraph;
        match self {
            Asset(ty) => {
                if changed && !is_up_down {
                    ty.update_preview(menuitem, true, true);
                    menu.is_changed = false;
                }
                BasicMenuResult::new()
            }
            CurrentData => {
                if !is_up_down && changed{
                    hub_room_set_by_result(None, ReloadType::ForcedUpdate);
                    menu.is_changed = false;
                }
                BasicMenuResult::new()
            }
            OutfitDataFile => {
                let emblem = menu.god_mode;
                let limit = if emblem { 3 } else { 5 };
                let previous = menu.loaded_data.profile;
                if !is_up_down && menu.is_changed { UnitAssetMenuData::reload_unit(ReloadPreview::LoadedData, true, None); }
                if r_l_press(true, false, true) {
                    menu.loaded_data.profile = (limit + previous - 1) % limit;
                    set_detail_box(None, None, Some(self.get_body(menuitem)), ProfileItem(Profile::from_index(menu.loaded_data.profile)).get_icon(menuitem).get_icon());
                    BasicMenuResult::se_cursor()
                }
                else if r_l_press(false, true, true) {
                    menu.loaded_data.profile = (limit + previous + 1) % limit;
                    set_detail_box(None, None, Some(self.get_body(menuitem)), ProfileItem(Profile::from_index(menu.loaded_data.profile)).get_icon(menuitem).get_icon());
                    BasicMenuResult::se_cursor()
                }
                else {
                    let left = Pad::is_trigger(NpadButton::l_key());
                    let right = Pad::is_trigger(NpadButton::r_key());
                    if left || right {
                        let box_state =
                            if left { menu.loaded_data.equipment_box_state.get_previous() }
                            else { menu.loaded_data.equipment_box_state.get_next() };

                        EquipmentBoxMode::LoadData(box_state).update();
                        menu.loaded_data.equipment_box_state = box_state;
                        BasicMenuResult::se_cursor()
                    }
                    else {
                        if menu.is_changed { UnitAssetMenuData::reload_unit(ReloadPreview::LoadedData, false, None); }
                        BasicMenuResult::new()
                    }
                }
            }
            FlagMenuItem(flag) => { flag.custom_call(menuitem) }
            ScaleMenuItem(scale_index) => {
                let i = *scale_index as usize;
                let trigger = pad.npad_state.buttons.y();
                let left = r_l_press(true, false, trigger);
                let right = r_l_press(false, true, trigger);
                if left || right {
                    if scale_change_value(*scale_index as i32, right, false) {
                        let result = UnitAssetMenuData::get_result();
                        let menu_data = UnitAssetMenuData::get_preview();
                        for x in 0..16 {
                            if menu_data.scale_preview[x] == 0 { menu_data.scale_preview[x] = menu_data.original_scaling[x]; }
                            if menu_data.scale_preview[x] > 0 { result.scale_stuff[x] = menu_data.scale_preview[x] as f32 / 100.0; }
                        }
                        if photo {
                            for x in 0..16 { menu_data.preview_data.scale[x] = menu_data.scale_preview[x]; }
                            menuitem.set_decided(false);
                        }
                        else {
                            menuitem.set_decided(
                                (menu_data.preview_data.scale[i] == menu_data.scale_preview[i]) ||
                                    (menu_data.preview_data.scale[i] == 0 && menu_data.scale_preview[i] == menu_data.original_scaling[i])
                            );
                        }
                        menuitem.rebuild_text();
                        hub_room_set_by_result(None, ReloadType::Scale);
                        BasicMenuResult::se_cursor()
                    }
                    else { BasicMenuResult::se_miss() }
                }
                else { BasicMenuResult::new() }
            }
            RGBA { kind, color } => {
                let i = (4*kind + color) as usize;
                let preview = UnitAssetMenuData::get_preview();
                let trigger = pad.npad_state.buttons.y();
                let left = r_l_press(true, false, trigger);
                let right = r_l_press(false, true, trigger);
                if left || right {
                    let change = if left { -1 } else { 1 };
                    preview.color_preview[i] = ((preview.color_preview[i] as i32 + change) % 255) as u8;
                    if photo {
                        preview.preview_data.colors[*kind as usize].values[*color as usize] = preview.color_preview[i];
                    }
                    self.on_select(menuitem);
                    menuitem.rebuild_text();
                    BasicMenuResult::se_cursor()
                }
                else { BasicMenuResult::new() }
            }
            CurrentProfile => {
                if change_selected_profile() {
                    menuitem.rebuild_text();
                    EquipmentBoxMode::CurrentProfile.update();
                    set_detail_box(None, Some(get_current_profile_assignment_text().into()), None, None);
                    BasicMenuResult::se_cursor()
                }
                else { BasicMenuResult::new() }
            }
            ProfileItem(profile) => {
                if Pad::is_trigger(NpadButton::new().with_left(true)) {
                    let new = profile.left();
                    EquipmentBoxMode::ProfilePreview(new).update();
                    menuitem.menu_kind = ProfileItem(new);
                    menuitem.rebuild_text();
                    if let Some(data) = UnitAssetMenuData::get_current_asset_data() {
                        data.set_profile[menuitem.index as usize] = new.to_index() as i32;
                    }
                    return BasicMenuResult::se_cursor();
                }
                else if Pad::is_trigger(NpadButton::new().with_right(true)) {
                    let new = profile.right();
                    EquipmentBoxMode::ProfilePreview(new).update();
                    menuitem.menu_kind = ProfileItem(new);
                    menuitem.rebuild_text();
                    if let Some(data) = UnitAssetMenuData::get_current_asset_data() {
                        data.set_profile[menuitem.index as usize] = new.to_index() as i32;
                    }
                    return BasicMenuResult::se_cursor();
                }
                BasicMenuResult::new()
            }
            UnitName => {
                if r_l_press(true, false, true) | r_l_press(false, true, true) {
                    if let Some(unit) = UnitAssetMenuData::get_shop_unit() {
                        return
                        if unit.person.parent.index > 1 && unit.person.flag.value & 128 == 0 && unit.edit.name.is_some() {
                            let new_gender = if unit.edit.gender == 1 || unit.edit.gender == 2 { 0 } else { unit.person.gender };
                            unit.edit.set_gender(new_gender);
                            menuitem.rebuild_text();
                            if let Some(char_name) = GameObject::find("CharacterName")
                                .and_then(|v| v.get_component_in_children::<TextMeshProUGUI>(true))
                            {
                                char_name.set_text(unit.get_name(), true);
                            }
                            BasicMenuResult::se_decide()
                        }
                        else { BasicMenuResult::se_miss() };
                    }
                }
                BasicMenuResult::new()
            }
            PresetAppearance => {
                if menu.is_changed && !is_up_down {
                    if UnitAssetMenuData::is_photo_graph() {
                        hub_room_set_by_result(None, ReloadType::All);
                        menu.is_changed = false;
                    }
                    else {
                        UnitAssetMenuData::reload_unit(ReloadPreview::Preset(menuitem.hash as usize), true, None);
                    }
                }
                let left = Pad::is_trigger(NpadButton::l_key());
                let right = Pad::is_trigger(NpadButton::r_key());
                if left || right {
                    let box_state = menu.loaded_data.equipment_box_state.get_preset_appearance(right);
                    EquipmentBoxMode::LoadData(box_state).set_preset_appearance(menuitem.hash);
                    menu.loaded_data.equipment_box_state = box_state;
                    BasicMenuResult::se_cursor()
                }
                else { BasicMenuResult::new() }
            }
            _ => { BasicMenuResult::new() }
        }
    }
}
fn get_current_profile_assignment_text() -> String {
    let help = MenuText::get_help(1).unwrap();
    let selected = UnitAssetMenuData::get_preview().selected_profile;
    let profiles_used =
        UnitAssetMenuData::get_current_asset_data()
            .map(|d|
                d.set_profile.iter().enumerate().filter(|(i, v)| **v == selected && *i < 3)
                    .map(|(i, _)| i)
                    .collect::<Vec<usize>>()
            )
            .unwrap_or(Vec::new());
    if profiles_used.is_empty() { format!("{}\n<color=\"yellow\">Profile is not assigned.</color>", help) } else {
        let emblem = UnitAssetMenuData::get().god_mode;
        let mut profile_str = String::new();
        profiles_used.iter().for_each(|i| {
            if !profile_str.is_empty() { profile_str += ", "; } else { profile_str += get_profile_name(*i as i32, emblem).to_string().as_str() }
        });
        format!("{}\nAssigned to: {}", help, profile_str)
    }
}
fn set_unit_name(unit: &Unit, value: &Il2CppString, _: OptionalMethod) {
    if value.is_null() { return; }
    let str = value.to_string();
    if str.len() == 0 { return; }
    unit.edit.set_name(value);
    if let Some(hub) = HubAccessoryRoom::get_instance().and_then(|c| c.get_child())
        .map(|p| p.cast_mut::<HubAccessoryShopSequence>()){
        hub.change_root.unit_name.set_text(unit.get_name(), true);
    }
}
pub fn get_current_profile_name() -> &'static Il2CppString {
    let emblem = UnitAssetMenuData::get().god_mode;
    let selection = UnitAssetMenuData::get_preview().selected_profile;
    get_profile_name(selection, emblem)
}
pub fn get_profile_name(index: i32, emblem: bool) -> &'static Il2CppString {
    match index {
        0 => { Mess::get(PROFILE_MID[0]) },
        1 => { if emblem { Mess::get(PROFILE_MID[3]) } else { MenuTextCommand::Engage.get() }},
        2 => { Mess::get("MID_SAVEDATA_SEQ_HUB") }
        3 => { format!("{} 1", MenuTextCommand::Alt).into() }
        4 => { format!("{} 2", MenuTextCommand::Alt).into() }
        _ => { unreachable!() }
    }
}

pub fn get_random_scaling(ty: i32, rng: &Random) -> i32 {
    match ty {
        0 => { 75 + rng.get_value(75) }
        1 => { 85 + rng.get_value(30) }
        2..4 => { 90 + rng.get_value(30) }
        4..9|10..16 => { 80 + rng.get_value(40) }
        9 => { 75 + rng.get_value(225) }
        _ => { 0 }
    }
}
pub fn scale_change_value(index: i32, increase: bool, speed_up: bool) -> bool {
    let preview = UnitAssetMenuData::get_preview();
    let v = preview.scale_preview[index as usize];
    if (v == 1000 && increase) || (v <= 1 && !increase) {
        if v <= 1 { preview.scale_preview[index as usize] = 1; }
        else if v >= 1000 { preview.scale_preview[index as usize] = 1000; }
        false
    }
    else {
        let increase_by = if speed_up { 10 } else { 1 };
        let value = if increase { v + increase_by } else { v - increase_by } as i32;
        let new_value = crate::clamp_value(value, 1, 1000) as u16;
        preview.scale_preview[index as usize] = new_value;
        true
    }
}
fn delete_face_item(menu_item: &mut CustomAssetMenuItem, _: OptionalMethod) {
    let path = format!("{}{}", THUMB_DIR, menu_item.name);
    let idx = menu_item.hash;
    if let Ok(_) = fs::remove_file(path.as_str()) {
        let load_face = &mut UnitAssetMenuData::get().loaded_data.load_face;
        if let Some(face) = load_face.iter().position(|s| s.index == idx as usize)
        {
            load_face.remove(face);
            FaceThumbnail::remove(format!("LOAD_{}", idx), true);
        }
        menu_item.menu.full_menu_item_list.clear();
        if load_face.len() == 0 {
            ProfileSettings.create_menu_items(menu_item.menu);
            menu_item.menu.menu_kind = ProfileSettings;
        }
        else {
            FaceSelection.create_menu_items(menu_item.menu);
            menu_item.menu.menu_kind = FaceSelection;
        }
        menu_item.menu.rebuild_menu();
    }
}