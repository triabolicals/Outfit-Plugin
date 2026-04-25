use std::ops::BitXor;
use engage::gamedata::{Gamedata, GodData, PersonData};
use engage::unit::{Gender};
use engage::menu::BasicMenuItemAttribute;
use engage::mess::Mess;
use unity::prelude::Il2CppString;
use crate::{get_current_profile_name, get_outfit_data, left_right_enclose, AssetType, CustomAssetMenu, CustomAssetMenuItem, EquipmentBoxMode, EquipmentBoxPage, UnitAssetMenuData};
use crate::data::items::{AssetFlag, CustomMenuItem, Profile};
use crate::menu::icons::CustomMenuIcon;
use super::*;
use crate::data::room::hub_room_set_by_result;
use engage::sequence::photograph::*;
use crate::localize::{MenuText, MenuTextCommand};
use crate::room::ReloadType;

#[repr(C)]
#[derive(PartialEq, Copy, Clone)]
pub enum CustomAssetMenuKind {
    MainShop,
    ProfileSelection,
    ProfileSettings,
    ShopBody((u8, bool)),
    ShopAcc(u8),
    ShopAoc(u8),
    ShopMount(u8),
    ClassBodySelection((u8, bool)),
    Head,
    Hair,
    VoiceSelection,
    ScaleMenu,
    ColorKindSelection,
    ColorSelection(u8),
    RGBAMenu(u8),
    PresetAppearanceMenu(bool),
    Rig,
    Personal,
    LoadData,
    PauseList,
    ItemList,
}
impl CustomAssetMenuKind {
    pub fn can_facial(&self) -> bool {
        match self {
            ScaleMenu | RGBAMenu(_) | ProfileSelection|ProfileSettings => { false }
            _ => { self.get_right().is_none() && self.get_left().is_none() }
        }
    }
    pub fn to_index(&self) -> i32 {
        match self {
            MainShop => 0,
            ProfileSelection => 1,
            ProfileSettings => 2,
            Head => 3,
            Hair => 4,
            VoiceSelection => 5,
            ColorKindSelection => 6,
            ScaleMenu => 7,
            Rig => 9,
            Personal => 10,
            LoadData => 11,
            PresetAppearanceMenu(_) => 12,
            PauseList => 13,
            ItemList => 14,
            ShopBody((kind, alt)) => { clamp_menu_index_value(if *alt { 104 } else { 100 }, *kind, 4) },    //1020
            ShopAcc(kind) => clamp_menu_index_value(110, *kind, 5),
            ShopMount(kind) => clamp_menu_index_value(120, *kind, 5),
            ShopAoc(kind) => clamp_menu_index_value(130, *kind, 4),
            ColorSelection(kind) => clamp_menu_index_value(140, *kind, 8),
            RGBAMenu(kind) => clamp_menu_index_value(150, *kind, 8),
            ClassBodySelection((class, alt)) => { clamp_menu_index_value(if *alt { 210 } else { 170 }, *class, 40) },
        }
    }
    pub fn from_index(value: i32) -> Self {
        match value {
            0 => MainShop,
            1 => ProfileSelection,
            2 => ProfileSettings,
            3 => Head,
            4 => Hair,
            5 => VoiceSelection,
            6 => ColorKindSelection,
            7 => ScaleMenu,
            9 => Rig,
            10 => Personal,
            11 => LoadData,
            13 => PauseList,
            14 => ItemList,
            100..108 => ShopBody(((value as u8 - 100) % 4, value >= 104)),
            110..115 => ShopAcc(value as u8 - 110),
            120..125 => ShopMount(value as u8 - 120),
            130..135 => ShopAoc(value as u8 - 130),
            140..148 => ColorSelection(value as u8 - 140),
            150..158 => RGBAMenu(value as u8 - 150),
            170..250 => ClassBodySelection(((value as u8 - 170) % 40, value >= 210)),
            _ => { unreachable!() },
        }
    }
    pub fn get_help_index(&self, is_menu_item: bool) -> i32 {
        let idx = self.to_index();
        if is_menu_item && idx >= 100 {
            match idx {
                170|210 => 1027,
                140..148 => { 1000 + idx }
                171..210|211..250 => 1028,
                _ => { 1020 + ((idx - 100 ) / 10) }
            }
        }
        else {
            match idx {
                100..108 => { 1100 + (idx % 4) }
                150..158 => { 1150 }
                170 | 210 => 1027,
                171..210 | 211..250 => 1028,
                _ => { 1000 + idx }
            }
        }
    }
    pub fn get_previous(&self) -> Option<Self> {
        match self {
            ClassBodySelection((_, alt)) => { Some(ShopBody((1, *alt))) }
            ProfileSelection|ScaleMenu|ColorKindSelection|Head|Hair|VoiceSelection => { Some(MainShop) }
            ColorSelection(_) => { Some(ColorKindSelection) }
            RGBAMenu(k) => { Some(ColorSelection(*k)) }
            LoadData|FaceSelection => Some(ProfileSettings),
            MainShop => None,
            PresetAppearanceMenu(alt) => {
                if UnitAssetMenuData::get_preview().preview_data.flag & 128 != 0 {
                    if *alt { Some(PresetAppearanceMenu(false)) }
                    else { Some(MainShop) }
                }
                else { Some(MainShop) }
            }
            _ => { Some(MainShop) }
        }
    }
    pub fn get_right(&self) -> Option<Self> {
        match self {
            ClassBodySelection((class, alt)) => {
                let db = get_outfit_data();
                let count = if UnitAssetMenuData::get_gender(*alt) == 2 { db.list.job_count.1 } else { db.list.job_count.0 } as u8;
                let new_class = (*class + count + 1) % count;
                Some(ClassBodySelection((new_class, *alt)))
            }
            ShopBody((kind, alt)) => {
                let alt = *alt;
                if UnitAssetMenuData::get_preview().preview_data.flag & 128 != 0 {
                    if *kind == 3 {
                        UnitAssetMenuData::get_preview().update_dress_gender = true;
                        Some(ShopBody((0, !alt)))
                    }
                    else { Some(ShopBody(((*kind + 4 + 1) % 4, alt))) }
                }
                else { Some(ShopBody(((*kind + 4 + 1) % 4, false))) }
            }
            ShopMount(kind) => { Some(ShopMount((*kind + 5 + 1) % 5)) }
            ShopAoc(kind) => { Some(ShopAoc((*kind + 4 + 1) % 4)) }
            ShopAcc(kind) => { Some(ShopAcc((*kind + 5 + 1) % 5)) }
            ColorKindSelection => { Some(ColorSelection(0)) }
            ColorSelection(k) => { if *k < 7 { Some(ColorSelection(*k + 1)) } else { Some(ColorKindSelection) } }
            PresetAppearanceMenu(alt) => {
                if UnitAssetMenuData::get_preview().preview_data.flag & 128 != 0 { Some(PresetAppearanceMenu(!(*alt))) }
                else { None }
            }
            _ => { None }
        }
    }
    pub fn get_left(&self) -> Option<Self> {
        match self {
            ClassBodySelection((class, alt)) => {
                let db = get_outfit_data();
                let count = if UnitAssetMenuData::get_gender(*alt) == 2 { db.list.job_count.1 } else { db.list.job_count.0 } as u8;
                let new_class = (*class + count - 1) % count;
                Some(ClassBodySelection((new_class, *alt)))
            }
            ShopBody((kind, alt)) => {
                let alt = *alt;
                if UnitAssetMenuData::get_preview().preview_data.flag & 128 != 0 {
                    if *kind == 0 {
                        UnitAssetMenuData::get_preview().update_dress_gender = true;
                        Some(ShopBody((3, !alt)))
                    }
                    else { Some(ShopBody(((*kind + 4 - 1) % 4, alt))) }
                }
                else { Some(ShopBody(((*kind + 4 - 1) % 4, alt))) }
            }
            ShopMount(kind) => { Some(ShopMount((*kind + 5 - 1) % 5)) }
            ShopAoc(kind) => { Some(ShopAoc((*kind + 4 - 1) % 4)) }
            ShopAcc(kind) => { Some(ShopAcc((*kind + 5 - 1) % 5)) }
            ColorKindSelection => { Some(ColorSelection(7)) }
            ColorSelection(k) => { if *k == 0 { Some(ColorKindSelection) } else { Some(ColorSelection(*k - 1)) } }
            _ => { None }
        }
    }
    pub fn get_menu_item_name(&self) -> Option<&'static Il2CppString> {
        let idx = self.to_index() + 1000;
        match self {
            ProfileSettings => { Some(MenuTextCommand::Settings.get()) }
            ShopBody((0, _))  => { Some(Mess::get("MID_Hub_amiibo_Accessory_Trade")) }
            ShopBody((1, _)) => { Some(MenuTextCommand::Class.get()) }
            ClassBodySelection((class, alt)) => {
                let db = get_outfit_data();
                let set = if UnitAssetMenuData::get_gender(*alt) == 2 { &db.list.job_f } else { &db.list.job_m };
                set.get(*class as usize).map(|v| Mess::get(v.label).to_string().into())
            }
            ShopBody((2, _)) => { Some(MenuTextCommand::Engage.get())}
            ShopBody((3, _)) => { Some(Mess::get("MID_ProfileCard_Stamp_Others")) }
            Head => { Some(Mess::get("MID_Hub_Mascot_Accessories_Head")) }
            ShopAcc(0) => { Some(Mess::get("MID_Hub_Mascot_Accessories_Parts")) }
            VoiceSelection => { Some(MenuTextCommand::Voice.get()) }
            Personal => { Some(MenuTextCommand::Personal.get()) }
            ItemList => { Some(MenuTextCommand::Weapons.get()) }
            _ => { Some(MenuText::get_command(idx)) }
        }
    }
    pub fn get_save_select_index(&self) -> Option<usize> {
        match self {
            MainShop => { Some(0) },
            ProfileSelection => { Some(1) }
            ProfileSettings => { Some(2) }
            ShopBody((kind, false)) => { Some(3+*kind as usize) }
            ShopBody((kind, true)) => { Some(7+*kind as usize) }
            Head => { Some(11) }
            Hair => { Some(12) }
            VoiceSelection => { Some(13) }
            ScaleMenu => { Some(14) }
            ShopAoc(kind) => { Some(15 + *kind as usize) }
            ShopAcc(kind) => { Some(19 + *kind as usize) }
            ShopMount(kind) => { Some(24 + *kind as usize) }
            ColorKindSelection => { Some(29) }
            ColorSelection(color_kind) => { Some(30 + *color_kind as usize) }
            Rig => { Some(48) }
            Personal => { Some(49) }
            PauseList => { Some(15) }
            ItemList => { Some(16) }
            _ => { None }
        }
    }
    pub fn create_menu_items(&self, this: &mut CustomAssetMenu) {
        let db = get_outfit_data();
        let preview = UnitAssetMenuData::get_preview();
        let female = UnitAssetMenuData::get_gender(false) == 2;
        match self {
            LoadData => {
                let item = CustomAssetMenuItem::new_type(CurrentData);
                item.name = MenuText::get_command(7);
                this.full_menu_item_list.add(item);
                UnitAssetMenuData::get().loaded_data.loaded_data.iter().for_each(|x|{
                    let item = CustomAssetMenuItem::new_type(OutfitDataFile);
                    item.name = x.get_filename().into();
                    this.full_menu_item_list.add(item);
                });
            }
            FaceSelection => {
                UnitAssetMenuData::get().loaded_data.load_face.iter().for_each(|x|{
                    let item = CustomAssetMenuItem::new_type(FaceThumb);
                    item.name = x.file_name.as_str().into();
                    item.hash = x.index as i32;
                    this.full_menu_item_list.add(item);
                });
            }
            MainShop => {
                if UnitAssetMenuData::is_photo_graph() {
                    for x in [ShopBody((0, false)), Head, Hair, Rig, ShopAcc(0), ColorKindSelection, ScaleMenu, Personal, PresetAppearanceMenu(false)]{
                        this.full_menu_item_list.add(CustomAssetMenuItem::new_menu2(x));
                    }
                    if let Some(p) = PhotographTopSequence::get_photograph_sequence() {
                       if p.dispos_manager.current_dispos_info.weapon_data_list.len() > 1 {
                           this.full_menu_item_list.add(CustomAssetMenuItem::new_menu2(ItemList));
                       }
                       if p.dispos_manager.current_dispos_info.pause_data_list.len() > 1 {
                           this.full_menu_item_list.add(CustomAssetMenuItem::new_menu2(PauseList));
                       }
                   }
                }
                else {
                    this.full_menu_item_list.add(CustomAssetMenuItem::new_type(CurrentProfile));
                    for x in [
                        ProfileSelection, ProfileSettings, ShopBody((0, false)), Head, Hair, Rig,
                        ShopAcc(0), VoiceSelection, ShopAoc(0), ShopMount(0), ColorKindSelection, ScaleMenu, Personal, PresetAppearanceMenu(false)]
                    {
                        this.full_menu_item_list.add(CustomAssetMenuItem::new_menu2(x));
                    }
                    if !UnitAssetMenuData::get().god_mode { this.full_menu_item_list.add(CustomAssetMenuItem::new_type(UnitName)); }
                }
            }
            PauseList => {
                if let Some(info) = PhotographTopSequence::get_photograph_sequence().map(|p|&p.dispos_manager.current_dispos_info) {
                    let mid = info.current_pause_data.map(|v|{ v.mid.to_string() });
                    info.pause_data_list.iter().for_each(|x|{
                        let item = CustomAssetMenuItem::new_type(Pause);
                        item.name = x.get_name();
                        item.decided = Some(x.mid.to_string()) == mid;
                        this.full_menu_item_list.add(item);
                    });
                }
                if this.full_menu_item_list.len() < 2 { this.full_menu_item_list.add(CustomAssetMenuItem::new_type(NoItem)); }
            }
            ItemList => {
                if let Some(info) = PhotographTopSequence::get_photograph_sequence().map(|p| &p.dispos_manager.current_dispos_info) {
                    let hash = info.weapon_data.map(|v| v.parent.hash).unwrap_or(0);
                    info.weapon_data_list.iter().for_each(|x|{
                        let item = CustomAssetMenuItem::new_type(Item);
                        item.name = x.get_name();
                        item.decided = hash == x.parent.hash;
                        this.full_menu_item_list.add(item);
                    });
                }
                if this.full_menu_item_list.len() < 2 { this.full_menu_item_list.add(CustomAssetMenuItem::new_type(NoItem)); }
            }
            ProfileSelection => {
                if let Some(data) = UnitAssetMenuData::get_current_asset_data() {
                    for x in 0..3 { this.full_menu_item_list.add(CustomAssetMenuItem::new_type(ProfileItem(Profile::from_index(data.set_profile[x])))); }
                }
            }
            PresetAppearanceMenu(cross) => {
                let female = UnitAssetMenuData::get_gender(*cross) == 2;
                let photo = UnitAssetMenuData::is_photo_graph();
                db.dress.personal.iter().enumerate()
                    .filter(|(_, d)| female == d.is_female && ((!d.morph == photo) || !photo))
                    .for_each(|(i, d)|{
                        if !d.morph {
                            let item =  CustomAssetMenuItem::new_type(PresetAppearance);
                            item.hash = i as i32;
                            let index = (d.index & 0xFFFFFF) << 1;
                            item.padding = if d.emblem { 1 } else { 0 } | index;
                            item.name = d.get_menu_name();
                            this.full_menu_item_list.add(item);
                        }

                    });
            }
            Personal => {
                let female = UnitAssetMenuData::get_current_dress_gender() == 2;
                if let Some(name) =
                    PersonData::try_get_hash(preview.person).and_then(|person| { person.name.map(|v| v.to_string()) })
                        .or_else(||{ GodData::try_get_hash(preview.person).map(|god|{ god.mid.to_string() }) })
                {
                    let set = if female { &db.list.char_f } else { &db.list.char_m };
                    let alt = name.replace("MGID_", "MPID_");
                    if let Some(group) = set.iter().find(|c| c.label == name || c.label == alt) {
                        group.list.iter().for_each(|c| {
                            this.full_menu_item_list.add(CustomAssetMenuItem::new_asset2(c, group.label));
                        });
                    }
                }
                if this.full_menu_item_list.is_empty() { this.full_menu_item_list.add(CustomAssetMenuItem::new_type(NoItem)); }
            }
            ProfileSettings => {
                [FlagMenuItem(AssetFlag::RandomAppearance), FlagMenuItem(AssetFlag::EnableColor), FlagMenuItem(AssetFlag::EnableScaling),
                    FlagMenuItem(AssetFlag::EngageOutfit), FlagMenuItem(AssetFlag::EnableCrossDressing), FlagMenuItem(AssetFlag::EngagedAnimation),
                    FlagMenuItem(AssetFlag::EnableBattleAccessories), FlagMenuItem(AssetFlag::UseFaceThumbnail), Data(AssetDataMode::Export), Data(AssetDataMode::Import),
                    FlagMenuItem(AssetFlag::ViewMode)
                ].into_iter().for_each(|v|{
                    this.full_menu_item_list.add(CustomAssetMenuItem::new_type(v));
                });
            }
            ShopBody((0, alt)) => {    // Unit (Same Gender)
                let female = UnitAssetMenuData::get_gender(*alt) == 2;
                db.list.add_menu_items(AssetType::Body, female, true, false, &db.labels, this.full_menu_item_list);
            }
            ShopBody((1, alt))  => {
                let female = UnitAssetMenuData::get_gender(*alt) == 2;
                let class_count = if female { db.list.job_count.1 } else { db.list.job_count.0 } as usize;
                let set = if female { &db.list.job_f } else { &db.list.job_m };
                for x in 0..class_count{
                    this.full_menu_item_list.add(CustomAssetMenuItem::new_menu3(ClassBodySelection((x as u8, *alt)),  Mess::get(set[x].label)));
                }
            }
            ClassBodySelection((class, alt)) => {
                let index = *class as usize;
                let current = UnitAssetMenuData::get_current_unit_hash(AssetType::Body);
                let female = UnitAssetMenuData::get_gender(*alt) == 2;
                let set = if female { &db.list.job_f } else { &db.list.job_m };
                if let Some(class) = set.get(index) {
                    class.list.iter().filter(|x| x.kind == AssetType::Body)
                        .for_each(|a|{
                            if let Some(body) = db.hashes.body.get(&a.hash){
                                let name = db.labels.get_suffix_name(body.as_str()).unwrap_or(body.to_string().trim_start_matches("uBody_").into());
                                let item = CustomAssetMenuItem::new_asset(AssetType::Body, a.hash, name, current == a.hash, preview.original_assets[0] == a.hash);
                                this.full_menu_item_list.add(item);
                            }
                        });
                }
                if this.full_menu_item_list.is_empty() { this.full_menu_item_list.add(CustomAssetMenuItem::new_type(NoItem)); }
            }
            ShopBody((2, alt))  => {
                let female = UnitAssetMenuData::get_gender(*alt) == 2;
                db.list.engaged.iter()
                    .filter(|x| x.female == female)
                    .for_each(|a|{
                        let item = CustomAssetMenuItem::new_asset3(&a, &db.labels, true);
                        item.name = Mess::get(a.label.as_str());
                        this.full_menu_item_list.add(item);
                    });
            }
            ShopBody((3, alt)) => {
                let female = UnitAssetMenuData::get_gender(*alt) == 2;
                db.list.add_menu_items(AssetType::Body, female, false, true, &db.labels, this.full_menu_item_list);
            }
            Head => { db.list.add_menu_items(AssetType::Head, female, true, true, &db.labels, this.full_menu_item_list); }
            Hair => { db.list.add_menu_items(AssetType::Hair, female,true, true, &db.labels, this.full_menu_item_list); }
            Rig => {
                let current = UnitAssetMenuData::get_current_unit_hash(AssetType::Rig);
                let original = preview.original_assets[15];
                db.hashes.rigs.iter().for_each(|(h, n)|{
                    let name = n.trim_start_matches("uRig_").into();
                    let item = CustomAssetMenuItem::new_asset(AssetType::Rig, *h, name, current == *h, original == *h);
                    this.full_menu_item_list.add(item);
                });
            }
            ShopAcc(kind) => { db.list.add_menu_items(AssetType::Acc(*kind), false, true, true, &db.labels, this.full_menu_item_list); }
            ShopAoc(page) => {
                if UnitAssetMenuData::is_unit_info() && UnitAssetMenuData::get_unit().is_some() {
                    let help = if UnitAssetMenuData::get_person_flag() & 8 != 0 { "Replace Face" } else { "Capture Face" };
                    add_key_help(KeyHelpButton::Minus, help);
                }
                let female = db.get_dress_gender_hash(preview.preview_data.ubody).map(|v| v == Gender::Female).unwrap_or(female);
                db.list.add_menu_items(AssetType::AOC(*page), female, true, true, &db.labels, this.full_menu_item_list);
            }
            ShopMount(mount) => {
                db.list.add_menu_items(AssetType::Mount(*mount), false, true, true, &db.labels, this.full_menu_item_list);
            }
            ScaleMenu => {
                this.full_menu_item_list.add(CustomAssetMenuItem::new_type(FlagMenuItem(AssetFlag::EnableScaling)));
                let preview = UnitAssetMenuData::get_preview();
                for x in 0..16 {
                    if preview.scale_preview[x] == 0 {
                        let v = preview.preview_data.scale[x];
                        if v > 0 && v < 1000 { preview.scale_preview[x] = v; }
                        else { preview.scale_preview[x] = preview.original_scaling[x]; }
                    }
                    this.full_menu_item_list.add(CustomAssetMenuItem::new_type(ScaleMenuItem(x as u8)));
                }
            }
            ColorKindSelection => {
                this.full_menu_item_list.add(CustomAssetMenuItem::new_type(FlagMenuItem(AssetFlag::EnableColor)));
                for x in 0..8 { this.full_menu_item_list.add(CustomAssetMenuItem::new_menu2(ColorSelection(x))); }
            }
            ColorSelection(page) => {
                this.full_menu_item_list.add(CustomAssetMenuItem::new_type(ResetColor(*page)));
                this.full_menu_item_list.add(CustomAssetMenuItem::new_menu2(RGBAMenu(*page)));
                let kind = *page;
                let preview = UnitAssetMenuData::get_preview();
                db.list.color_presets.iter()
                    .filter(|x| x.colors[kind as usize ] != 0)
                    .for_each(|x| {
                        let hash = x.colors[kind as usize];
                        let mut selected = true;
                        let mut original = true;
                        for x in 0..3 {
                            let r = ((hash >> 8*x) & 255) as u8;
                            if preview.preview_data.colors[kind as usize].values[x] != r { selected = false; }
                            if preview.original_color[4*kind as usize + x] != r { original = false; }
                        }
                        let name = x.get_name();
                        let item = CustomAssetMenuItem::new_asset(AssetType::ColorPreset(kind), x.colors[kind as usize], name, selected, original);
                        this.full_menu_item_list.add(item);
                    });
            }
            VoiceSelection => {
                db.list.add_menu_items(AssetType::Voice, false, true, true, &db.labels, this.full_menu_item_list);
            }
            RGBAMenu(page) => {
                let k = *page;
                this.full_menu_item_list.add(CustomAssetMenuItem::new_type(ResetColor(*page)));
                for x in 0..4 { this.full_menu_item_list.add(CustomAssetMenuItem::new_type(RGBA {kind: k, color: x})); }
                let preview = UnitAssetMenuData::get_preview();
                if preview.preview_data.colors[k as usize].has_color() {
                    for x in 0..3 {
                        preview.color_preview[4*k as usize + x] = preview.preview_data.colors[k as usize].values[x];
                    }
                }
            }
            _ => { this.full_menu_item_list.add(CustomAssetMenuItem::new(-1, -1)); }
        }
        if this.full_menu_item_list.len() == 0 { this.full_menu_item_list.add(CustomAssetMenuItem::new_type(NoItem)); }
        else {
            if let Some(index) = self.get_save_select_index(){
                if this.selects[index].index == 0 {
                    if let Some(pos) = this.full_menu_item_list.iter().position(|v| v.decided)
                        .or_else(|| this.full_menu_item_list.iter().position(|v| v.original))
                    {
                        this.selects[index].index = pos as i32;
                        this.selects[index].scroll = pos as i32;
                    }
                }

            }
        }
    }
    pub fn b_call(&self) {
        let reload_type =
            match self {
                ScaleMenu => {
                    hub_room_set_by_result(None, ReloadType::All);
                    UnitAssetMenuData::get().control.setup(false, true);
                    return;
                }
                FaceSelection => {
                    CustomAssetMenu::toggle_ui();
                    disable_key_help(KeyHelpButton::Minus);
                    UnitAssetMenuData::get().loaded_data.release_faces();
                    return;
                }
                ShopMount(_) => { ReloadType::ForcedUpdate }
                ColorKindSelection|ColorSelection(_)|RGBAMenu(_) => { ReloadType::ForcedUpdate }
                ShopAoc(_)|ShopAcc(_) => { ReloadType::All }
                PresetAppearanceMenu(_)|LoadData => {
                    UnitAssetMenuData::get().loaded_data.selected_index = None;
                    ReloadType::ForcedUpdate
                }
                ProfileSettings|VoiceSelection => { ReloadType::NoUpdate }
                _ => { ReloadType::All }
            };
       hub_room_set_by_result(None, reload_type );
    }
    pub fn build_attribute(&self, emblem: bool) -> BasicMenuItemAttribute {
        match self {
            ShopMount(_)|ShopAoc(_)|PresetAppearanceMenu(_) => if emblem { BasicMenuItemAttribute::Hide } else { BasicMenuItemAttribute::Enable },
            _ => BasicMenuItemAttribute::Enable,
        }
    }
}
impl CustomMenuItem for CustomAssetMenuKind {
    fn get_icon(&self, _menu_item: &CustomAssetMenuItem) -> CustomMenuIcon {
        match self{
            MainShop|ProfileSelection => CustomMenuIcon::KeyItem,
            ProfileSettings => CustomMenuIcon::TimeCrystal,
            PresetAppearanceMenu(_)|ShopBody(_)|ClassBodySelection(_)|Personal => CustomMenuIcon::Clothes,
            Rig => CustomMenuIcon::Body,
            VoiceSelection => CustomMenuIcon::Talk,
            ShopAcc(_) => CustomMenuIcon::AccFace,
            ShopAoc(_) => CustomMenuIcon::SolaTail,
            ShopMount(_) => CustomMenuIcon::Horse,
            ScaleMenu => CustomMenuIcon::StarBlank,
            ColorKindSelection|ColorSelection(_)|RGBAMenu(_) => CustomMenuIcon::Star,
            Head => { CustomMenuIcon::Head }
            Hair => { CustomMenuIcon::Hair }
            LoadData => { CustomMenuIcon::Satchel }
            ItemList => { CustomMenuIcon::Weapon }
            PauseList => { CustomMenuIcon::SolaTail }
        }
    }
    fn get_equipment_box_type(&self, _menu_item: &CustomAssetMenuItem) -> EquipmentBoxMode {
        match self{
            Rig|ShopBody(_)|ClassBodySelection(_)|Hair|Head => EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Assets),
            ShopAcc(_) => EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::AccessoryAssets),
            ShopAoc(_)|VoiceSelection => EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::AOCAnimations),
            ShopMount(_) => EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::RideMounts),
            ScaleMenu => EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Scaling(0)),
            ColorSelection(kind)|RGBAMenu(kind) => EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Color(*kind)),
            ColorKindSelection => EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Color(0)),
            _ => EquipmentBoxMode::CurrentProfile,
        }
    }

    fn get_name(&self, _menuitem: &CustomAssetMenuItem) -> &'static Il2CppString {
        match self {
            ProfileSettings => { MenuTextCommand::Settings.get() }
            ShopBody(_)  => { Mess::get("MID_Hub_amiibo_Accessory_Trade") }
            ClassBodySelection((class, alt)) => {
                let db = get_outfit_data();
                if UnitAssetMenuData::get_gender(*alt) == 2 { &db.list.job_f }
                else { &db.list.job_m }.get(*class as usize).map(|v| Mess::get(v.label)).unwrap()
            }
            ShopAcc(_) => { Mess::get("MID_Hub_Mascot_Accessories_Parts") }
            VoiceSelection => { MenuTextCommand::Voice.get() }
            Head => { Mess::get("MID_Hub_Mascot_Accessories_Head") }
            Personal => { MenuTextCommand::Personal.get() }
            _ => { self.get_menu_item_name().unwrap() }
        }
    }
    fn get_detail_box_name(&self, _menu_item: &CustomAssetMenuItem) -> Option<&'static Il2CppString> {
        match self {
            ProfileSelection => { Some(get_current_profile_name()) }
            _ => { Some(self.get_name(_menu_item)) }
        }
    }
    fn get_help(&self, _menuitem: &CustomAssetMenuItem) -> &'static Il2CppString {
        let idx = self.get_help_index(true);
        MenuText::get_help(self.get_help_index(true)).unwrap_or(format!("Menu Help #{}", idx).into())
    }
    fn get_body(&self, menu_item: &CustomAssetMenuItem) -> &'static Il2CppString {
        match self {
            Personal|LoadData => { menu_item.menu_kind.get_body(menu_item) }
            ClassBodySelection((_, alt)) => {
                let page = if *alt { 6 } else { 2 };
                let count = if UnitAssetMenuData::get_flag() & 128 != 0 { 8 } else { 4 };
                left_right_enclose(
                &format!("{} ({}) [{}/{}]",
                         MenuTextCommand::Class.get(),
                         MenuTextCommand::get_gender(UnitAssetMenuData::get_gender(*alt) == 2),
                         page,
                         count)
                )
            }
            ShopAcc(kind) => {
                left_right_enclose(&format!("{} [{}/5]", MenuText::get_command(60 + *kind as i32), *kind + 1))
            }
            ShopMount(kind) => {
                left_right_enclose(&format!("{} [{}/5]", MenuText::get_command(70 + *kind as i32), *kind + 1))
            }
            ShopBody((kind, alt)) => {
                let page_count = if UnitAssetMenuData::get_flag() & 128 != 0 { 8 } else { 4 };
                let page = format!(" [{}/{}]", if *alt { *kind as i32 + 4 } else { *kind as i32 } + 1, page_count);
                let mut name =
                    match kind {
                        1 => { MenuTextCommand::Class.get() }
                        2 => { MenuTextCommand::Engage.get() }
                        3 => { Mess::get("MID_ProfileCard_Stamp_Others") }
                        _ => { Mess::get("MID_ProfileCard_Stamp_Unit") }
                    }.to_string();
                if UnitAssetMenuData::get_flag() & 128 != 0 {
                    if UnitAssetMenuData::get_gender(*alt) == 2 {
                        name.push_str(" (Female)");
                    }
                    else { name.push_str(" (Male)") }
                }
                left_right_enclose(&format!("{}{}", name, page))
            }
            ShopAoc(kind) => {
                let db = get_outfit_data();
                let mut body = format!("{} ({})",
                   MenuText::get_command(80 + *kind as i32),
                   MenuTextCommand::get_gender(db.get_aoc_gender_hash(*kind as i32, menu_item.hash) == Some(Gender::Female))
                );
                body.push_str(&format!(" [{}/4]", *kind +1).as_str());
                left_right_enclose(&body)
            },
            PresetAppearanceMenu(alt) => {
                if *alt {
                    left_right_enclose(
                        &format!("{} ({})",
                                 MenuTextCommand::Data,
                                 MenuTextCommand::get_gender(UnitAssetMenuData::get_gender(*alt) == 2)
                        )
                    )
                }
                else {
                    if menu_item.padding & 1 != 0 { MenuTextCommand::Emblem.get() }
                    else { MenuTextCommand::Units.get() }
                }
            }
            RGBAMenu(kind) => MenuText::get_command(1140 + *kind as i32),
            ColorSelection(kind) => left_right_enclose(&format!("{} [{}/8]", MenuText::get_command(1140 + *kind as i32), *kind+1)),
            MainShop => "".into(),
            _ => { self.get_name(menu_item) }
        }
    }
}

pub fn clamp_menu_index_value(offset: i32, value: u8, size: i32) -> i32 {
    let v = value as i32;
    if v < size && v >= 0 { offset + v } else { offset }
}