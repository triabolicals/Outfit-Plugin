use std::fs;
use engage::{
    app::{AccessoryData_Kinds, BasicDialog, BasicDialogItemNo, GameUserData, GameVariable, IAccessoryMenuItemMethods, IBasicDialogItem, IBasicDialogMethods, IBitField32, IBitField32Methods, IGameVariableMethods, IPhotographDisposInfo, IPhotographDisposInfoMethods, IPhotographDisposManager, IPhotographSequence, IRandom_2Methods, ISpriteAtlasManager_2, IUnit, IUnitEdit, Pad, Random_2},
    GameVariableManager,
    system::collections::generic::{IDictionary_2Methods, InsertionBehavior},
    app::IUnitEditMethods
};
use crate::{
    is_up_down_press, left_right_enclose, AssetType, THUMB_DIR,
    data::{items::Profile, room::hub_room_set_by_result},
    menu::{icons::CustomMenuIcon, items::{AssetFlag, CustomAssetMenuKind, CustomMenuItem}, *},
    localize::{MenuText, MenuTextCommand}, FACIAL_STATES
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
    RGBA(u8), // 100 + 4*kind + color
    EnableColor(u8),
    Menu(CustomAssetMenuKind),  // 1000 + menu index
    OutfitDataFile,
    CurrentData,
    PresetAppearance,
    Pause,
    Item,
    UnitInventorySubMenuItem,
    FaceThumb,
    Expression(u8),
}
impl CustomAssetMenuItemKind {
    pub fn can_facial(&self) -> bool {
        match self {
            OutfitDataFile|UnitName|CurrentProfile|ScaleMenuItem(_)|RGBA(_)|ProfileItem(_)|Expression(_) => false,
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
            CurrentData => 7,
            PresetAppearance => 8,
            Data(AssetDataMode::ExportPreview) => 9,
            ProfileItem(profile) => 10 + profile.to_index() as i32,
            FlagMenuItem(ty) => { 20 + ty.get_rel_index() }
            ResetColor(color) =>  30 + *color as i32,
            Asset(ty) => 40 + ty.to_index(),
            RGBA(kind) => 100 + (*kind as i32),
            EnableColor(kind) => 120 + (*kind as i32),
            ScaleMenuItem(ty) => 150 + *ty as i32,  // 300 -> 316
            Expression(kind) => 170 + *kind as i32,
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
            100..116=> {
                let offset = index - 100;
                RGBA(offset as u8)
            }
            120..136 => { EnableColor(index as u8 - 120) }
            150..166 => { ScaleMenuItem(index as u8 - 150) }
            170..174 => { Expression(index as u8 - 170) }
            1000..1500 => {
                let menu_offset = index - 1000;
                Menu(CustomAssetMenuKind::from_index(menu_offset))
            }
            -1 => Pause,
            -2 => Item,
            -4 => FaceThumb,
            _ => NoItem,
        }
    }
    pub fn on_select(&self, menu_item: CustomAssetMenuItem3) {
        if *self == UnitInventorySubMenuItem { return; }
        let help = self.get_help(menu_item);
        let menu_kind = menu_item.get_asset_menu().menu_kind();
        let body = menu_kind.get_body(menu_item);
        let icon = self.get_icon(menu_item);
        let name = self.get_detail_box_name(menu_item);
        set_detail_box(name, Some(help), Some(body), icon.get_icon());
        self.get_equipment_box_type(menu_item).update();
        let index = menu_item.get_index();
        match self {
            CurrentData => {
                let data = UnitAssetMenuData::get();
                data.loaded_data.selected_index = None;
                let box_state = data.loaded_data.equipment_box_state;
                EquipmentBoxMode::CurrentProfilePage(box_state).update();
                UnitAssetMenuData::set_reload(ReloadPreview::Full, true);
                set_detail_box(name, Some(help), Some(body), icon.get_icon());
            }
            OutfitDataFile => {
                let data = UnitAssetMenuData::get();
                let current = if index == 0 { None } else { Some(index-1) };
                if data.loaded_data.selected_index != current {
                    data.loaded_data.selected_index = current;
                    println!("Loaded Data Set to {}", current.unwrap());
                    UnitAssetMenuData::set_reload(ReloadPreview::LoadedData, true);
                    let result = UnitAssetMenuData::get_result();
                    let menu = UnitAssetMenuData::get();
                    if let Some(loaded) = menu.loaded_data.selected_index.and_then(|i| menu.loaded_data.loaded_data.get_mut(i as usize)) {
                        let flag = loaded.data.flag;
                        loaded.data.flag |= 193;
                        loaded.data.set_result(result, 2, false, false);
                        loaded.data.flag = flag;
                    }
                    let box_state = data.loaded_data.equipment_box_state;
                    EquipmentBoxMode::LoadData(box_state).update();
                }
                data.loaded_data.selected_index = current;
                let name = self.get_detail_box_name(menu_item);
                set_detail_box(name, Some(help), Some(body), Profile::from_index(data.loaded_data.profile).get_icon().get_icon());
            }
            Asset(ty) => {
                ty.update_box(menu_item);
                UnitAssetMenuData::set_reload(ReloadPreview::Asset, false);
            }
            RGBA(kind) => {
                let v2 = menu_item.value2();
                let pos = match v2 { 16 => 2, 17 => 3, 14 => 4, _ => { v2 % 8 + 1 } };
                let color_kind = (*kind % 16) as usize;
                EquipmentBoxMode::set_cursor(Some(pos));
                let preview = UnitAssetMenuData::get_preview();
                if !preview.preview_data.colors[color_kind].has_color() && color_kind < 8 {
                    for x in 0..3 {
                        let v = preview.original_color[4*color_kind + x];
                        preview.color_preview[color_kind*4+x] = v;
                    }
                }
                UnitAssetMenuData::set_reload(ReloadPreview::Color(color_kind as i32), false);
            }
            Menu(menu) => {
                match menu {
                    ShopBody(_) => EquipmentBoxMode::set_cursor(Some(1)),
                    Head => EquipmentBoxMode::set_cursor(Some(2)),
                    Hair => EquipmentBoxMode::set_cursor(Some(3)),
                    Rig => EquipmentBoxMode::set_cursor(Some(4)),
                    VoiceSelection => EquipmentBoxMode::set_cursor(Some(5)),
                    ColorSelection(16)|ColorPresets(_, 16) => {
                        EquipmentBoxMode::Hair.update();
                        EquipmentBoxMode::set_cursor(Some(2));
                    }
                    ColorSelection(17)|ColorPresets(_, 17)  => {
                        EquipmentBoxMode::Hair.update();
                        EquipmentBoxMode::set_cursor(Some(3));
                    }
                    ColorSelection(14)|ColorPresets(_, 14) => {
                        EquipmentBoxMode::Hair.update();
                        EquipmentBoxMode::set_cursor(Some(4));
                    }
                    ColorSelection(kind)| ColorPresets(_, kind) => {
                        let k = *kind % 8;
                        EquipmentBoxMode::set_color_cursor(Some(k as i32));
                    }
                    _ => EquipmentBoxMode::set_cursor(None),
                }
            }
            ResetColor(color_kind) => {
                let v2 = menu_item.value2();
                let pos = match v2 { 16 => 2, 17 => 3, 14 => 4, _ => { v2 % 8 + 1 } };
                let color_kind = *color_kind as usize;
                EquipmentBoxMode::set_cursor(Some(pos));
                UnitAssetMenuData::set_reload(ReloadPreview::ResetColor(color_kind as i32), false);
            }
            ScaleMenuItem(kind) => {
                let cursor_index = (*kind % 8) + 1;
                EquipmentBoxMode::set_cursor(Some(cursor_index as i32));
                UnitAssetMenuData::set_reload(ReloadPreview::ScalePreview(*kind as i32), false);
            }
            PresetAppearance => {
                EquipmentBoxMode::set_cursor(None);
                if UnitAssetMenuData::is_photo_graph(){
                    let db = get_outfit_data();
                    if let Some(appearance) = db.dress.personal.get(menu_item.value() as usize) {
                        UnitAssetMenuData::get().preview.preview_data.set_from_preset(appearance);
                    }
                }
                else {
                    let data = UnitAssetMenuData::get();
                    let box_state = data.loaded_data.equipment_box_state;
                    if box_state == EquipmentBoxPage::Flags {
                        data.loaded_data.equipment_box_state = EquipmentBoxPage::Assets;
                    }
                    EquipmentBoxMode::LoadData(data.loaded_data.equipment_box_state).set_preset_appearance(menu_item.value() );
                }
                UnitAssetMenuData::set_reload(ReloadPreview::Preset(menu_item.value() as usize), true);
            }
            Pause => {
                if let Some(dispos) = crate::photo::get_photosequence().map(|p| p.m_dispos_manager().m_current_dispos_info()){
                    let pause = dispos.get_pause_data_list().get(index);
                    if !pause.is_null() {
                        dispos.set_m_current_pause_data(pause);
                        dispos.set_up_pause();
                    }
                }
            }
            Item  => {
                if let Some(dispos) = crate::photo::get_photosequence().map(|p| p.m_dispos_manager().m_current_dispos_info()){
                    let item = dispos.get_weapon_data_list().get(index);
                    if !item.is_null() {
                        dispos.set_m_hold_weapon_data(item);
                        dispos.setup_weapon();
                    }
                }
            }
            Expression(kind) => {
                let k = *kind as usize;
                let v = UnitAssetMenuData::get_preview().preview_data.expression[k] as usize;
                let v = if v == 0 { k } else { v - 1 };
                hub_room_set_by_result(None, ReloadType::FacialPreview(v));
                EquipmentBoxMode::set_cursor(None);
            }
            _ => { EquipmentBoxMode::set_cursor(None); }
        }
    }
    pub fn build_attribute(&self) -> BasicMenuItem_Attribute {
        let emblem = UnitAssetMenuData::get_unit().is_none();
        let dvc = UnitAssetMenuData::get().is_dvc;
        let photo = UnitAssetMenuData::is_photo_graph();
        let hide =
        match self {
            Data(_) => { photo }
            FlagMenuItem(AssetFlag::EnableCrossDressing) => emblem,
            FlagMenuItem(AssetFlag::EngagedAnimation)|FlagMenuItem(AssetFlag::EngageOutfit) => emblem || photo,
            FlagMenuItem(AssetFlag::UseFaceThumbnail) => emblem && !UnitAssetMenuData::is_unit_info(),
            FlagMenuItem(AssetFlag::RandomAppearance) => emblem || !dvc || photo,
            UnitName|Menu(ShopMount(_))|Menu(ShopAoc(_))|Menu(PresetAppearanceMenu(_))  => emblem,
            Menu(EngagedBody(_)) => !emblem,
            _ => false,
        };
        if hide { BasicMenuItem_Attribute::hide() } else { BasicMenuItem_Attribute::enable() }
    }
}

impl CustomMenuItem for CustomAssetMenuItemKind {
    fn get_icon(&self, menu_item: CustomAssetMenuItem3) -> CustomMenuIcon {
        match self {
            Expression(_) => CustomMenuIcon::AccFace,
            RGBA(_)|ResetColor(_) => CustomMenuIcon::Color,
            ScaleMenuItem(_) => CustomMenuIcon::StarBlank,
            Asset(ty) => ty.get_icon(menu_item),
            CurrentProfile|EnableColor(_) => CustomMenuIcon::KeyItem,
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
    fn get_equipment_box_type(&self, menu_item: CustomAssetMenuItem3) -> EquipmentBoxMode {
        match self {
            Asset(asset) => asset.get_equipment_box_type(menu_item),
            FlagMenuItem(flag) => flag.get_equipment_box_type(menu_item),
            RGBA(kind)|ResetColor(kind) => {
                let v2 = menu_item.value2();
                match v2 {
                    14|16|17 => EquipmentBoxMode::Hair,
                    _ => EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Color(*kind))
                }
            },
            ResetColor(kind) => EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Color(*kind)),
            ScaleMenuItem(kind) => EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Scaling(*kind/8)),
            ProfileItem(profile) => EquipmentBoxMode::ProfilePreview(*profile),
            Menu(menu) => menu.get_equipment_box_type(menu_item),
            _ => EquipmentBoxMode::CurrentProfile,
        }
    }
    fn get_name(&self, menuitem: CustomAssetMenuItem3) -> unity::Il2CppString {
        let idx = self.to_index();
        let menu_index = menuitem.m_index();
        match self {
            UnitInventorySubMenuItem => { MenuTextCommand::Outfits.get() }
            PresetAppearance|Pause|Item => { menuitem.m_name() }
            OutfitDataFile|FaceThumb => { menuitem.m_name() }
            Menu(menu) => menu.get_name(menuitem),
            Asset(ty) => ty.get_name(menuitem),
            FlagMenuItem(flag) => flag.get_name(menuitem),
            ScaleMenuItem(scale_index) => {
                let i = *scale_index as usize;
                let preview = UnitAssetMenuData::get_preview();
                if preview.scale_preview[i] == 0 {
                    let v = preview.preview_data.scale[i] & 1023;
                    if v == 0 || v > 1000  { preview.scale_preview[i] = preview.original_scaling[i]; }
                    else { preview.scale_preview[i] = v; }
                }
                let v = preview.scale_preview[i];
                format!("{}: {}", MenuText::get_command(idx), v as f32 / 100.0).into()
            }
            EnableColor(_) => { "Enable Color".into() }
            ResetColor(kind) => {
                let kind = *kind as usize;
                let preview = UnitAssetMenuData::get_preview();
                if kind < 8 {
                    format!("Original: {} {} {}",  preview.original_color[4*kind],  preview.original_color[4*kind+ 1],  preview.original_color[4*kind+2]).into()
                }
                else { "..".into() }
            }
            RGBA(kind) => {
                let kind = (*kind % 16) as usize;
                let preview = UnitAssetMenuData::get_preview();
                format!("RGB: {} {} {}",  preview.color_preview[4*kind],  preview.color_preview[4*kind+ 1],  preview.color_preview[4*kind+2]).into()
            }
            CurrentProfile => {
                let emblem = UnitAssetMenuData::get().god_mode;
                let preview = UnitAssetMenuData::get_preview();
                format!("{}: {}", MenuText::get_command(1), get_profile_name(preview.selected_profile, emblem)).into()
            }
            ProfileItem(p) => {
                let emblem = UnitAssetMenuData::get().god_mode;
                let i = if menu_index == 1 && emblem { 3 } else { menu_index } as usize;
                format!("{}: {}", engage::app::Mess::get(PROFILE_MID[i]), p.get_name()).into()
            }
            UnitName => {
                if let Some(unit) = UnitAssetMenuData::get_shop_unit().filter(|v| !v.m_edit().m_name().is_null()){
                    format!("{} [{}]", unit.get_name(), MenuTextCommand::on_off(unit.m_edit().is_enable())).into()
                }
                else { "Custom Name".into() }
            }
            Data(data) => { data.get_name(menuitem) }
            NoItem => { "------".into() }
            Expression(kind) => {
                let k = *kind as usize;
                let v = UnitAssetMenuData::get_preview().preview_data.expression[k] as usize;
                if v == 0 || v > 13 { format!("{}: Default", FACIAL_STATES[k].0).into() }
                else { format!("{}: {}", FACIAL_STATES[k].0, FACIAL_STATES[v - 1].0).into() }
            }
            _ => { MenuText::get_command(idx) }
        }
    }
    fn get_detail_box_name(&self, menuitem: CustomAssetMenuItem3) -> Option<unity::Il2CppString> {
        let idx = self.to_index();
        match self {
            OutfitDataFile => {
                if let Some(select) = UnitAssetMenuData::get().loaded_data.get_selected_data() { select.path.file_stem().and_then(|x| x.to_str()).map(|x| x.into()) }
                else { None }
            }
            Asset(ty) => ty.get_detail_box_name(menuitem),
            FlagMenuItem(flag) => flag.get_detail_box_name(menuitem),
            EnableColor(kind) => { Some(format!("Enable: {}", MenuText::get_command(1140+ *kind as i32)).into()) }
            RGBA(_) => Some("RGB".into()),
            ProfileItem(profile) => { Some(profile.get_name()) }
            Menu(menu) => { menu.get_detail_box_name(menuitem) }
            CurrentProfile => { Some(get_current_profile_name()) }
            Data(item) => { item.get_detail_box_name(menuitem) }
            UnitName => { Some("Unit Name".into()) }
            PresetAppearance => { Some(menuitem.m_name()) }
            FaceThumb => { Some(menuitem.m_name().to_rust_string().trim_end_matches(".png").into()) }
            Expression(kind) => { Some(FACIAL_STATES[*kind as usize].0.into()) }
            _ => { Some(MenuText::get_command(idx)) }
        }
    }
    fn get_help(&self, menuitem: CustomAssetMenuItem3) -> unity::Il2CppString {
        let idx = self.to_index();
        match self {
            ResetColor(kind) => {
                let set = UnitAssetMenuData::get_set_color_str(*kind as i32);
                let original = UnitAssetMenuData::get_original_color_str(*kind as i32);
                if set != original { MenuTextCommand::A.to_right(MenuTextCommand::Confirm) }
                else { MenuText::get_help(31).unwrap() }
            }
            EnableColor(_) => { MenuTextCommand::A.insert_right("Toggle this color parameter.") }
            Asset(ty) => ty.get_help(menuitem),
            FlagMenuItem(flag) => flag.get_help(menuitem),
            RGBA(_) => {
                format!("{}/{}/{} Change RGB\n{} {} {}",
                  MenuTextCommand::LeftRight,
                  MenuTextCommand::LR,
                  MenuTextCommand::ZRZL,
                  MenuTextCommand::A.insert_right("Slow (Hold)"),
                  MenuTextCommand::X.to_right(MenuTextCommand::Random),
                  MenuTextCommand::Minus.to_right(MenuTextCommand::Reset)
                ).into()
            },
            ScaleMenuItem(scale_index) => {
                let i = *scale_index as usize;
                let preview = UnitAssetMenuData::get_preview();
                let original = preview.original_scaling[i] as f32 / 100.0;
                format!("{}\n{} {} {} {}",
                    MenuTextCommand::Original.insert_right(original),
                    MenuTextCommand::A.insert_right(if !menuitem.get_m_decided() { "Enable" } else { "Disable"}),
                    MenuTextCommand::LR.insert_right("Slow"), MenuTextCommand::X.to_right(MenuTextCommand::Random),
                    MenuTextCommand::Minus.insert_right(MenuTextCommand::Reset)
                ).into()
            },
            ProfileItem(_) => {
                let emblem = UnitAssetMenuData::get().god_mode;
                let i = if menuitem.get_index() == 1 && emblem { 3 } else { menuitem.get_index() };
                MenuText::get_help(10 + i).unwrap()
            },
            Menu(menu) => menu.get_help(menuitem),
            UnitName => {
                let help = MenuText::get_help(400).unwrap();
                if let Some(unit) = UnitAssetMenuData::get_shop_unit().filter(|u| !u.is_null()) {
                    let name = unit.m_edit().m_name();
                    if !name.is_null() {
                        let person = unit.get_person();
                        let help_idx = if person.get_flag().get_value() & 128 == 0 && person.index() > 1 { 401 } else { 400 };
                        format!("{}\nSet name: {}", MenuText::get_help(help_idx).unwrap(), name).into()
                    } else { format!("{}\nDefault: {}", help, engage::app::Mess::get_game_data_name(unit.get_pid())).into() }
                }
                else { help }
            }
            Expression(kind) => { MenuText::get_help_with_arg(1170, FACIAL_STATES[*kind as usize].0).unwrap() }
            CurrentProfile => { get_current_profile_assignment_text().into() }
            /*
            Anim(_) => {
                let mut help = format!("{}\n", MenuText::get_help(-5).unwrap());
                if UnitAssetMenuData::get_preview().anim_pause {
                    help.push_str(format!("{} {}", MenuTextCommand::A.insert_right("Play"), MenuTextCommand::Y.insert_right("Slow (Hold)")).as_str());
                }
                else { help.push_str(MenuTextCommand::A.insert_right("Play").to_string().as_str()); }
                help.push_str(MenuTextCommand::X.to_right(MenuTextCommand::Reset).to_string().as_str());
                help.into()
            }
            */
            _ => { MenuText::get_help(idx).unwrap_or(format!("MenuItemHelp #{}", idx).into()) }
        }
    }
    fn get_body(&self, menuitem:  CustomAssetMenuItem3) -> unity::Il2CppString {
        match self {
            CurrentData => { get_current_profile_name() }
            Asset(ty) => ty.get_body(menuitem),
            FlagMenuItem(flag) => flag.get_body(menuitem),
            ResetColor(kind)|RGBA(kind) => {
                let k = *kind as i32 % 16;
                MenuText::get_command(1140+ k)
            },
            Data(_) => MenuTextCommand::Data.get(),
            CurrentProfile|ProfileItem(_) => MenuText::get_command(1),
            OutfitDataFile => {
                if menuitem.m_index() == 0 { "".into() }
                else {
                    let emblem = UnitAssetMenuData::get().god_mode;
                    let saved_profile = UnitAssetMenuData::get().loaded_data.profile;
                    left_right_enclose(&format!("{}: {}", MenuTextCommand::Copy, get_profile_name(saved_profile, emblem)))
                }
            }
            PresetAppearance => {
                if menuitem.value2() & 1 != 0 { MenuTextCommand::Emblem.get() }
                else { MenuTextCommand::Personal.get() }
            }
            _ => "".into(),
        }
    }
    fn a_call(&self, menuitem: CustomAssetMenuItem3) -> BasicMenu_Result {
        let menu_item_idx = menuitem.get_index();
        match self {
            UnitInventorySubMenuItem => {
                let menu = menuitem.get_menu();
                let proc_parent = menu.get_super();
                let unit = SortieSelectionUnitManager::get_instance().m_unit();
                CustomAssetMenu::create_bind_unit_info(proc_parent, unit);
                BasicMenu_Result::close_decide()
            }
            EnableColor(kind) => {
                let preview = UnitAssetMenuData::get_preview();
                if preview.preview_data.colors[*kind as usize].values[3] != 0 {
                    preview.preview_data.colors[*kind as usize].values[3] = 0;
                    menuitem.set_m_decided(false);
                }
                else {
                    preview.preview_data.colors[*kind as usize].values[3] = 1;
                    menuitem.set_m_decided(true);
                }
                menuitem.rebuild_text();
                BasicMenu_Result::se_cursor()
            }
            Asset(ty) => ty.a_call(menuitem),
            ResetColor(kind) => {
                let i = (*kind as usize) % 16;
                let preview = UnitAssetMenuData::get_preview();
                for x in 0..4 {
                    if preview.preview_data.colors[i].values[x] != 0 {
                        preview.preview_data.colors[i].values[x] = 0;
                        preview.color_preview[4 * i + x] = preview.original_color[4 * i + x];
                    }
                }
                UnitAssetMenuData::set_reload(ReloadPreview::Color(*kind as i32), false);
                BasicMenu_Result::se_decide()
            }
            Data(data) => data.a_call(menuitem),
            Menu(menu) => {
                let asset_menu = menuitem.get_asset_menu();
                let kind = if *menu == MainShop { 0 } else { 1 };
                asset_menu.set_m_kind(AccessoryData_Kinds{value: kind});
                asset_menu.rebuild_menu(*menu, true);
                BasicMenu_Result::se_cursor()
            }
            UnitName => {
                /*
                if let Some(unit) = UnitAssetMenuData::get_shop_unit() {
                    UnitAssetMenuData::get().name_set = false;
                    let initial = unit.edit.name.or(Some(Mess::get_name(unit.person.pid)));
                    let header = Some(Mess::get("MID_GAMESTART_PLAYER_NAME_INPUT").to_string().into());
                    let sub_text = Some( "".into());
                    let limit = 20;
                    let action = Action1::new_with_method(Some(unit), set_unit_name);
                    engage::keyboard::SoftwareKeyboard::create_bind(menuitem.menu, limit, initial, header, sub_text, 0, Some(action));
                }

                 */
                BasicMenu_Result::se_cursor()
            }
            OutfitDataFile => {
                let data = UnitAssetMenuData::get();
                let result =
                if menu_item_idx > 0 {
                    let preview = UnitAssetMenuData::get_preview();
                    let db = get_outfit_data();
                    if let Some(new_data) = data.loaded_data.loaded_data.get( menu_item_idx as usize - 1){
                        let selected_profile = data.loaded_data.profile as usize;
                        let hash = data.preview.person;
                        if let Some(profile) = data.data.iter_mut().find(|x| x.person == hash )
                            .and_then(|unit_data| unit_data.profile.get_mut(selected_profile))
                        {
                            let current_dress = profile.ubody;
                            let current_dress_gender = db.get_dress_gender_hash(current_dress)
                                .or_else(||db.get_dress_gender_hash(data.preview.original_assets[0]))
                                .unwrap_or(engage::app::Gender::none());
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
                    BasicMenu_Result::se_decide()
                }
                else { BasicMenu_Result::se_miss() };
                menuitem.get_asset_menu().b_call();
                result
            }
            FaceThumb => {
                if let Some(unit) = UnitAssetMenuData::get_unit() {
                    if let Some(keys) = crate::capture::get_unit_face_keys(unit){
                        let table = engage::app::FaceThumbnail::s_face_thumb().m_cache_table();
                        let (found, sprite) = table.try_get_value(format!("LOAD_{}", menuitem.value()).into());
                        if found && !sprite.is_null(){
                            table.try_insert(keys.2.into(), sprite, InsertionBehavior::overwrite_existing());
                            UnitAssetMenuData::get().loaded_data.selected_index = Some(menuitem.value());
                            let key = format!("G_Face_{}", keys.0);
                            let name = menuitem.m_name().to_rust_string();
                            if !GameVariableManager::is_exist(key.as_str()) {
                                let user_data = GameUserData::get_instance();
                                let vars: GameVariable = user_data.get_variable();
                                vars.entry_2(key.as_str(), name);
                            }
                            else { GameVariableManager::set_string(key.as_str(), name); }
                            table.try_insert(keys.0.into(), sprite, InsertionBehavior::overwrite_existing());
                        }
                    }
                }
                menuitem.get_asset_menu().b_call();
                menuitem.get_asset_menu().toggle_ui();
                BasicMenu_Result::se_decide()
            }
            PresetAppearance => {
                let db = get_outfit_data();
                let preview = UnitAssetMenuData::get_preview();
                if let Some(appearance) = db.dress.personal.get(menuitem.value() as usize) {
                    preview.preview_data.set_from_preset(appearance);
                    BasicMenu_Result::se_decide()
                }
                else { BasicMenu_Result::se_miss() }
            }
            FlagMenuItem(flag) => { flag.a_call(menuitem) }
            ScaleMenuItem(kind) => {
                let preview = UnitAssetMenuData::get_preview();
                let v = preview.preview_data.scale[*kind as usize] & 1024 != 0;
                preview.preview_data.scale[*kind as usize] ^= 1024;
                menuitem.set_m_decided(!v);
                menuitem.rebuild_text();
                BasicMenu_Result::se_decide()
            }
            /*
            Anim(_) => {
                let v = UnitAssetMenuData::get_preview().anim_pause;
                UnitAssetMenuData::get_preview().anim_pause = !v;
                let speed = if v { 1.0 } else { 0.0 };
                let help = self.get_help(menuitem);
                let menu = menuitem.menu.menu_kind.clone();
                let body = menu.get_body(menuitem);
                let icon = self.get_icon(menuitem);
                let name = self.get_detail_box_name(menuitem);
                hub_room_set_by_result(None, ReloadType::BodyAnimSpeed(speed));
                set_detail_box(name, Some(help), Some(body), icon.get_icon());
                BasicMenu_Result::se_cursor()
            }

             */
            _ => { BasicMenu_Result::pass() }
        }
    }
    fn x_call(&self, menuitem: CustomAssetMenuItem3) -> BasicMenu_Result {
        match self {
            /*
            Anim(_) => {
                hub_room_set_by_result(None, ReloadType::BodyAnim(menuitem.hash as u32));
                BasicMenu_Result::se_cursor()
            }

             */
            ScaleMenuItem(scale_index) => {
                let i = *scale_index as usize;
                let preview = UnitAssetMenuData::get_preview();
                let random_value = get_random_scaling(i as i32, Random_2::get_game());
                if random_value > 0  {
                    preview.scale_preview[i] = random_value as u16;
                    preview.preview_data.scale[i] = random_value as u16;
                    menuitem.rebuild_text();
                    hub_room_set_by_result(None, ReloadType::Scale);
                    BasicMenu_Result::se_decide()
                }
                else { BasicMenu_Result::se_miss() }
            }
            RGBA(kind) => {
                let rng = Random_2::get_game();
                let i = (*kind % 16) as usize;
                let preview = UnitAssetMenuData::get_preview();
                for c in 0..3 {
                    let random_value = rng.get_value_2(255) as u8;
                    preview.color_preview[4 * i + c] = random_value;
                    preview.preview_data.colors[i].values[c] = random_value;
                }
                UnitAssetMenuData::set_reload(ReloadPreview::Color(i as i32), false);
                menuitem.rebuild_text();
                BasicMenu_Result::se_decide()
            }
            /*
            Asset(AssetType::Body) => {
                if !UnitAssetMenuData::get().god_mode && !UnitAssetMenuData::is_photo_graph() {
                    UnitAssetMenuData::get_preview().preview_data.break_body = menuitem.value();
                    menuitem.rebuild_text();
                    // menuitem.menu.full_menu_item_list.iter_mut().for_each(|v|{v.rebuild_text(); });
                    BasicMenu_Result::se_decide()
                }
                else { BasicMenu_Result::pass() }
            }

             */
            _ => { BasicMenu_Result::pass() }
        }
    }
    fn minus_call(&self, menuitem: CustomAssetMenuItem3) -> BasicMenu_Result {
        match self {
            ScaleMenuItem(scale_index) => {
                let i = *scale_index as usize;
                let preview = UnitAssetMenuData::get_preview();
                if preview.scale_preview[i] != preview.original_scaling[i] { preview.scale_preview[i] = preview.original_scaling[i]; }
                if UnitAssetMenuData::is_photo_graph()  {
                    preview.preview_data.scale[i] = preview.scale_preview[i] | (preview.preview_data.scale[i] & 1024);
                }
                else { preview.preview_data.scale[i] &= 1024; }
                UnitAssetMenuData::set_reload(ReloadPreview::ScalePreview(*scale_index as i32), false);
                menuitem.rebuild_text();
                BasicMenu_Result::se_decide()
            }
            RGBA(kind) => {
                let i = (*kind % 16) as usize;
                let preview = UnitAssetMenuData::get_preview();
                let mut changed = false;
                for c in 0..3 {
                    if preview.original_color[4 * i + c] != preview.color_preview[4 * i + c] {
                        changed = true;
                        preview.color_preview[4 * i + c] = preview.original_color[4 * i + c];
                        preview.preview_data.colors[i].values[c] = 0;
                    }
                }
                if changed {
                    UnitAssetMenuData::set_reload(ReloadPreview::Color((*kind % 16) as i32), false);
                    menuitem.rebuild_text();
                    BasicMenu_Result::se_decide()
                }
                else { BasicMenu_Result::se_miss() }
            }
            PresetAppearance => {
                let preview = UnitAssetMenuData::get_preview();
                let flags = preview.preview_data.flag;
                preview.preview_data = PlayerOutfitData::new_with_flag(flags);
                UnitAssetMenuData::set_reload(ReloadPreview::Preset(menuitem.value() as usize), true);
                BasicMenu_Result::se_decide()
            }
            Asset(AssetType::AOC(_)) => {
                if !UnitAssetMenuData::get().god_mode && UnitAssetMenuData::is_unit_info() {
                    let use_thumbnail = UnitAssetMenuData::get_person_flag() & 8 != 0;
                    crate::capture::capture_unit_info(menuitem.get_menu(), true, use_thumbnail);
                    BasicMenu_Result::se_cursor()
                } else { BasicMenu_Result::se_miss() }
            }
            FaceThumb|OutfitDataFile => {
                let action = engage::system::Action::new(
                    menuitem.into(),
                    if self.to_index() == -4 { delete_face_item_method_info() } else { delete_outfit_data_method_info() }.into()
                );
                let yes = engage::app::YesMenuItem::new(action);
                yes.set_m_text(engage::app::Mess::get("MID_MENU_YES"));
                let no = BasicDialogItemNo::new();
                let list = List_1::<engage::app::BasicMenuItem>::new_2(2);
                unsafe {
                    list.add(yes.cast());
                    list.add(no.cast());
                }
                let dialog = BasicDialog::create_basic_dialog_bind(menuitem.get_menu(), list);
                let message = format!("Delete '{}'?", menuitem.m_name());
                dialog.set_text(message);
                BasicMenu_Result::se_cursor()
            }
            _ => { BasicMenu_Result::pass() }
        }
    }
    fn custom_call(&self, menuitem: CustomAssetMenuItem3) -> BasicMenu_Result {
        let menu = UnitAssetMenuData::get();
        match self {
            /*
            Anim(_) => {
                if UnitAssetMenuData::get_preview().anim_pause {
                    let speed = if Pad::is_button(NpadButton::y_key()) { 0.09 } else { 0.0 };
                    hub_room_set_by_result(None, ReloadType::BodyAnimSpeed(speed));
                }
                BasicMenu_Result::pass()
            }

             */
            Asset(ty) => {
                if menu.reload_type.is_some() && !is_up_down_press() {
                    ty.update_model(menuitem);
                    menu.reload_type = None;
                }
                BasicMenu_Result::pass()
            }
            OutfitDataFile => {
                let emblem = menu.god_mode;
                let limit = if emblem { 3 } else { 5 };
                let previous = menu.loaded_data.profile;
                let l = Pad::is_trigger(NpadButton::l());
                let r = Pad::is_trigger(NpadButton::r());
                let left = Pad::is_trigger(NpadButton::left());
                let right = Pad::is_trigger(NpadButton::right());
                if left || right {
                    menu.loaded_data.profile = (limit + previous + if l { -1 } else { 1 }) % limit;
                    set_detail_box(None, None, Some(self.get_body(menuitem)), ProfileItem(Profile::from_index(menu.loaded_data.profile)).get_icon(menuitem).get_icon());
                    BasicMenu_Result::se_cursor()
                }
                else if l || r {
                    let box_state =
                        if r { menu.loaded_data.equipment_box_state.get_next() }
                        else { menu.loaded_data.equipment_box_state.get_previous() };
                    EquipmentBoxMode::LoadData(box_state).update();
                    menu.loaded_data.equipment_box_state = box_state;
                    BasicMenu_Result::se_cursor()
                }
                else { BasicMenu_Result::pass() }
            }
            FlagMenuItem(flag) => { flag.custom_call(menuitem) }
            ScaleMenuItem(scale_index) => {
                let i = *scale_index as usize;
                let menu_data = UnitAssetMenuData::get_preview();
                let key =
                    Pad::is_button(NpadButton::left()) as i32 +
                        ((Pad::is_button(NpadButton::right()) as i32) << 1) +
                        ((Pad::is_trigger(NpadButton::l()) as i32) << 2) +
                        ((Pad::is_trigger(NpadButton::r()) as i32) << 3);
                if key > 0 && ( key & (key - 1) == 0){
                    let previous = menu_data.scale_preview[i] & 1023;
                    let fast = Pad::is_button(NpadButton::y());
                    let next = scale_change_value(*scale_index as i32, key & 10 != 0, fast);
                    if previous == next { BasicMenu_Result::se_miss() }
                    else {
                        menu_data.preview_data.scale[i] = next | (menu_data.preview_data.scale[i] & 1024);
                        menuitem.rebuild_text();
                        UnitAssetMenuData::set_reload(ReloadPreview::ScalePreview(i as i32), false);
                        BasicMenu_Result::se_cursor()
                    }
                }
                else { BasicMenu_Result::pass() }
            }
            RGBA(kind) => {
                let k = (*kind % 16) as usize;
                let preview = UnitAssetMenuData::get_preview();
                let mut new_values = [0u8; 3];
                let trigger =  Pad::is_repeat(NpadButton::a());
                let keys = [(NpadButton::left(), NpadButton::right()), (NpadButton::l(), NpadButton::r()), (NpadButton::zl(), NpadButton::zr())];
                let amount = if Pad::is_button(NpadButton::y()) { 5 } else { 1 };
                let mut update = false;
                for x in 0..3 {
                    let (a, b) = &keys[x];
                    let i = 4*k+x;
                    let l = if trigger { Pad::is_trigger(*a) } else { Pad::is_button(*a) };
                    let r = if trigger { Pad::is_trigger(*b) } else { Pad::is_button(*b) };
                    if l == r {
                        new_values[x] = preview.color_preview[i];
                        continue;
                    }
                    update = true;
                    let change = amount * if l { -1 } else { 1 };
                    new_values[x] = ((preview.color_preview[i] as i32 + change) % 255) as u8;
                }
                if update {
                    for x in 0..3 {
                        preview.color_preview[4*k+x] = new_values[x];
                        preview.preview_data.colors[k].values[x] = new_values[x];
                    }
                    menuitem.rebuild_text();
                    self.on_select(menuitem);
                    hub_room_set_by_result(None, ReloadType::ColorScale);
                    BasicMenu_Result::se_cursor()
                }
                else { BasicMenu_Result::pass() }
            }
            CurrentProfile => {
                if change_selected_profile() {
                    menuitem.rebuild_text();
                    EquipmentBoxMode::CurrentProfile.update();
                    set_detail_box(None, Some(get_current_profile_assignment_text().into()), None, None);
                    BasicMenu_Result::se_cursor()
                }
                else { BasicMenu_Result::pass() }
            }
            ProfileItem(profile) => {
                if Pad::is_trigger(NpadButton::left()) {
                    let new = profile.left();
                    EquipmentBoxMode::ProfilePreview(new).update();
                    menuitem.set_menu_item_kind(ProfileItem(new));
                    menuitem.rebuild_text();
                    if let Some(data) = UnitAssetMenuData::get_current_asset_data() {
                        data.set_profile[menuitem.get_index() as usize] = new.to_index() as i32;
                    }
                    return BasicMenu_Result::se_cursor();
                }
                else if Pad::is_trigger(NpadButton::right()) {
                    let new = profile.right();
                    EquipmentBoxMode::ProfilePreview(new).update();
                    menuitem.set_menu_item_kind(ProfileItem(new));
                    menuitem.rebuild_text();
                    if let Some(data) = UnitAssetMenuData::get_current_asset_data() {
                        data.set_profile[menuitem.get_index() as usize] = new.to_index() as i32;
                    }
                    return BasicMenu_Result::se_cursor();
                }
                BasicMenu_Result::pass()
            }
            UnitName => {
                if Pad::is_trigger(NpadButton::left()) || Pad::is_trigger(NpadButton::right()) {
                    if let Some(unit) = UnitAssetMenuData::get_shop_unit() {
                        return
                        if unit.get_person().index() > 1 && unit.get_person().get_flag().m_value() & 128 == 0 && !unit.m_edit().m_name().is_null() {
                            let new_gender = if unit.m_edit().is_enable() { engage::app::Gender::none() } else { unit.get_person().get_gender() };
                            unit.m_edit().set_gender(new_gender);
                            menuitem.rebuild_text();
                            let name = menuitem.get_asset_menu().unit_name();
                            if !name.is_null() {
                                name.set_text_2(unit.get_name(), true);
                                BasicMenu_Result::se_decide()
                            }
                            else { BasicMenu_Result::se_miss() }
                        }
                        else { BasicMenu_Result::se_miss() };
                    }
                }
                BasicMenu_Result::pass()
            }
            PresetAppearance => {
                let left = Pad::is_trigger(NpadButton::l());
                let right = Pad::is_trigger(NpadButton::r());
                if left || right {
                    let box_state = menu.loaded_data.equipment_box_state.get_preset_appearance(right);
                    EquipmentBoxMode::LoadData(box_state).set_preset_appearance(menuitem.value());
                    menu.loaded_data.equipment_box_state = box_state;
                    BasicMenu_Result::se_cursor()
                }
                else { BasicMenu_Result::pass() }
            }
            Expression(kind) => {
                let preview = UnitAssetMenuData::get_preview();
                let l = Pad::is_trigger(NpadButton::left());
                let r = Pad::is_trigger(NpadButton::right());
                if l || r && r != l {
                    let mut v = (preview.preview_data.expression[*kind as usize] + if l { 13 } else { 1 } ) % 14;
                    if v == (*kind + 1) { v = (v + if l { 13 } else { 1 } ) % 14; }
                    preview.preview_data.expression[*kind as usize] = v;
                    menuitem.rebuild_text();
                    self.on_select(menuitem);
                    BasicMenu_Result::se_cursor()
                }
                else { BasicMenu_Result::pass() }
            }
            _ => { BasicMenu_Result::pass() }
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
pub fn get_current_profile_name() -> unity::Il2CppString {
    let emblem = UnitAssetMenuData::get().god_mode;
    let selection = UnitAssetMenuData::get_preview().selected_profile;
    get_profile_name(selection, emblem)
}
pub fn get_profile_name(index: i32, emblem: bool) -> unity::Il2CppString {
    match index {
        0 => { engage::app::Mess::get(PROFILE_MID[0]) },
        1 => { if emblem { engage::app::Mess::get(PROFILE_MID[3]) } else { MenuTextCommand::Engage.get() }},
        2 => { engage::app::Mess::get("MID_SAVEDATA_SEQ_HUB") }
        3 => { format!("{} 1", MenuTextCommand::Alt).into() }
        4 => { format!("{} 2", MenuTextCommand::Alt).into() }
        _ => { unreachable!() }
    }
}

pub fn get_random_scaling(ty: i32, rng: Random_2) -> i32 {
    match ty {
        0 => { 75 + rng.get_value_2(75) }
        1 => { 85 + rng.get_value_2(30) }
        2..4 => { 90 + rng.get_value_2(30) }
        4..9|10..16 => { 80 + rng.get_value_2(40) }
        9 => { 75 + rng.get_value_2(225) }
        _ => { 0 }
    }
}
pub fn scale_change_value(index: i32, increase: bool, speed_up: bool) -> u16 {
    let preview = UnitAssetMenuData::get_preview();
    let v = preview.scale_preview[index as usize] & 1023;
    let increase_by = if speed_up { 10 } else { 1 };
    let value = if increase { v + increase_by } else { v - increase_by } as i32;
    let new_value = crate::clamp_value(value, 1, 1000) as u16;
    preview.scale_preview[index as usize] = new_value;
    new_value
}
#[unity::callback]
fn delete_face_item(menu_item: CustomAssetMenuItem3, _: unity::OptionalMethod) {
    let path = format!("{}{}", THUMB_DIR, menu_item.m_name());
    let idx = menu_item.value();
    if let Ok(_) = fs::remove_file(path.as_str()) {
        let load_face = &mut UnitAssetMenuData::get().loaded_data.load_face;
        if let Some(face) = load_face.iter().position(|s| s.index == idx as usize)
        {
            load_face.remove(face);
            let key = format!("LOAD_{}", idx);
            let table = engage::app::FaceThumbnail::s_face_thumb().m_cache_table();
            let (found, sprite) = table.try_get_value(key.as_str().into());
            if found && !sprite.is_null() {
                table.remove(key.as_str().into());
                engage::unity_engine::Object_2::destroy_2(sprite);
            }
        }
        let menu = menu_item.get_asset_menu();
        menu.set_next(Some(if load_face.len() == 0 { ProfileSettings } else { FaceSelection }));
    }
}
#[unity::callback]
fn delete_outfit_data(menu_item: CustomAssetMenuItem3, _: unity::OptionalMethod) {
    let list = &mut UnitAssetMenuData::get().loaded_data.loaded_data;
    let name = menu_item.m_name().to_rust_string();
    if let Some(pos) = list.iter().position(|x| x.get_filename() == name) {
        let file = list.remove(pos);
        if fs::remove_file(file.path).is_ok() {
            let menu = menu_item.get_asset_menu();
            menu.set_next(Some(if list.len() == 0 { ProfileSettings } else { LoadData }));
        }
    }
}