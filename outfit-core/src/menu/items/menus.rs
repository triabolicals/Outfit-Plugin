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
            LoadData => Some(ProfileSettings),
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
                let count = if UnitAssetMenuData::get_gender(*alt) == 2 { db.list.class_female.len() } else { db.list.class_male.len() } as u8;
                let new_class = (*class + count + 1) % count;
                Some(ClassBodySelection((new_class, *alt)))
            }
            ShopBody((kind, alt)) => {
                let alt = *alt;
                if UnitAssetMenuData::get_preview().preview_data.flag & 128 != 0 {
                    if *kind == 3 { Some(ShopBody((0, !alt))) }
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
                let count = if UnitAssetMenuData::get_gender(*alt) == 2 { db.list.class_female.len() } else { db.list.class_male.len() } as u8;
                let new_class = (*class + count - 1) % count;
                Some(ClassBodySelection((new_class, *alt)))
            }
            ShopBody((kind, alt)) => {
                let alt = *alt;
                if UnitAssetMenuData::get_preview().preview_data.flag & 128 != 0 {
                    if *kind == 0 { Some(ShopBody((3, !alt))) }
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
                let set = if UnitAssetMenuData::get_gender(*alt) == 2 { &db.list.class_female } else { &db.list.class_male };
                set.get(*class as usize).map(|v| format!("{} ({})", Mess::get(v.class_label.as_str()), v.list.len()).into())
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
                let current = UnitAssetMenuData::get_current_unit_hash(AssetType::Body);
                if let Some(icon) =
                    PersonData::try_get_hash(preview.person).and_then(|person| { person.unit_icon_id.map(|v| v.to_string()) })
                        .or_else(||{ GodData::try_get_hash(preview.person).and_then(|god|{ god.unit_icon_id.map(|v| v.to_string()) })})
                {
                    let id = icon.split_at(3).0;
                    let mut hashes = vec![];
                    if female { &db.list.character_body_list.female } else { &db.list.character_body_list.male }
                        .iter()
                        .filter(|v| icon.contains(&v.label.name) || db.try_get_asset(AssetType::Body, v.hash).is_some_and(|b| b.contains(id)))
                        .for_each(|body|{
                            if !hashes.contains(&body.hash) {
                                let item = CustomAssetMenuItem::new_asset(AssetType::Body, body.hash, body.label.get(), current == body.hash, preview.original_assets[0] == body.hash);
                                hashes.push(body.hash);
                                this.full_menu_item_list.add(item);
                            }
                        });
                    let current = UnitAssetMenuData::get_current_unit_hash(AssetType::Head);
                    db.list.head_list.iter()
                        .filter(|v| icon.contains(&v.label.name) || db.try_get_asset(AssetType::Head, v.hash).is_some_and(|b| b.contains(id)))
                        .for_each(|body|{
                            if !hashes.contains(&body.hash) {
                                let item = CustomAssetMenuItem::new_asset(AssetType::Head, body.hash, body.label.get(), current == body.hash, preview.original_assets[1] == body.hash);
                                hashes.push(body.hash);
                                this.full_menu_item_list.add(item);
                            }
                        });
                    db.list.hair_list.iter()
                        .filter(|v| icon.contains(&v.label.name) || db.try_get_asset(AssetType::Hair, v.hash).is_some_and(|b| b.contains(id)))
                        .for_each(|body|{
                            if !hashes.contains(&body.hash) {
                                let item = CustomAssetMenuItem::new_asset(AssetType::Hair, body.hash, body.label.get(), current == body.hash, preview.original_assets[1] == body.hash);
                                hashes.push(body.hash);
                                this.full_menu_item_list.add(item);
                            }

                        });
                    for x in 0..4 {
                        let i = if x == 0 { 0 } else { x + 1};
                        let asset_type = AssetType::Acc(i as u8);
                        let current = UnitAssetMenuData::get_current_unit_hash(asset_type);
                        db.list.acc[x].iter().filter(|v| db.try_get_asset(asset_type, v.hash).is_some_and(|b| b.contains(id)) && !hashes.contains(&v.hash))
                            .for_each(|a|{
                                let hash = a.hash;
                                let name = db.try_get_asset(asset_type, a.hash).unwrap().trim_start_matches("uAcc_");
                                let name = name.split_once("_").map(|v| v.1.into()).unwrap_or(name.into());
                                if i == 0 {
                                    for ii in 0..2 {
                                        let ty = AssetType::Acc(ii as u8);
                                        let current = UnitAssetMenuData::get_current_unit_hash(ty);
                                        let item = CustomAssetMenuItem::new_asset(ty, hash, name, hash == current, hash == preview.original_assets[5 + ii]);
                                        this.full_menu_item_list.add(item);
                                    }
                                }
                                else { this.full_menu_item_list.add(CustomAssetMenuItem::new_asset(asset_type, hash, name, current == hash, preview.original_assets[5+i] == hash)); }
                            })
                    };
                    if female { &db.list.aoc_info_f} else { &db.list.aoc_info_m }.iter().enumerate().for_each(|(i, a)|{
                        let asset_type = AssetType::AOC(i as u8);
                        let current = UnitAssetMenuData::get_current_unit_hash(asset_type);

                        a.iter().filter(|v| db.try_get_asset(asset_type, **v).is_some_and(|b| b.contains(id)))
                            .for_each(|a|{
                                let hash = *a;
                                let name = db.try_get_asset(asset_type, *a).unwrap().into();
                                let item = CustomAssetMenuItem::new_asset(asset_type, hash, name, current == hash, preview.original_assets[10+i] == hash);
                                this.full_menu_item_list.add(item);
                            })
                    });
                }
                if this.full_menu_item_list.is_empty() { this.full_menu_item_list.add(CustomAssetMenuItem::new_type(NoItem)); }
            }
            ProfileSettings => {
                [FlagMenuItem(AssetFlag::RandomAppearance), FlagMenuItem(AssetFlag::EnableColor), FlagMenuItem(AssetFlag::EnableScaling),
                    FlagMenuItem(AssetFlag::EngageOutfit), FlagMenuItem(AssetFlag::EnableCrossDressing), FlagMenuItem(AssetFlag::EngagedAnimation),
                    FlagMenuItem(AssetFlag::EnableBattleAccessories), Data(AssetDataMode::Export), Data(AssetDataMode::Import),
                    FlagMenuItem(AssetFlag::ViewMode)
                ].into_iter().for_each(|v|{
                    this.full_menu_item_list.add(CustomAssetMenuItem::new_type(v));
                });
            }
            ShopBody((0, alt)) => {    // Unit (Same Gender)
                let current = UnitAssetMenuData::get_current_unit_hash(AssetType::Body);
                let female = UnitAssetMenuData::get_gender(*alt) == 2;
                let db = get_outfit_data();
                let set = if female { &db.list.character_body_list.female } else { &db.list.character_body_list.male };
                set.iter().for_each(|x|{
                    let item = CustomAssetMenuItem::new_asset(AssetType::Body, x.hash, x.label.get(), current == x.hash, preview.original_assets[0] == x.hash);
                    this.full_menu_item_list.add(item);
                });
            }
            ShopBody((1, alt))  => {
                let female = UnitAssetMenuData::get_gender(*alt) == 2;
                if female { &db.list.class_female } else { &db.list.class_male }.iter().enumerate().for_each(|(i, data)| {
                    this.full_menu_item_list.add(CustomAssetMenuItem::new_menu3(ClassBodySelection((i as u8, *alt)),  Mess::get(data.class_label.as_str())));
                });
            }
            ClassBodySelection((class, alt)) => {
                let index = *class as usize;
                let current = UnitAssetMenuData::get_current_unit_hash(AssetType::Body);
                let female = UnitAssetMenuData::get_gender(*alt) == 2;
                let set_main = if female { &db.list.class_female[index] } else { &db.list.class_male[index] };
                set_main.list.iter().for_each(|data| {
                    if let Some(body) = db.try_get_asset(AssetType::Body, data.hash).map(|v| v.to_string()) {
                        let name =
                            if data.flags & 1 != 0 {
                                if let Some(name1) = db.labels.body.iter().find(|x| body.ends_with(*x.0)).or_else(|| db.labels.body.iter().find(|x| body.contains(*x.0))) {
                                    if data.flags & 2 != 0 { format!("{} ({})", name1.1.get(), Mess::get("MID_SYS_BasicPosition")).into() } else if data.flags & 4 != 0 {
                                        if let Some(name) = db.labels.suffixes.iter().find(|x| body.ends_with(*x.0)) { format!("{} ({})", name1.1.get(), name.1.get()).into() } else { name1.1.get() }
                                    }
                                    else { name1.1.get() }
                                }
                                else { body.trim_start_matches("uBody_").into() }
                            }
                            else { db.try_get_suffix(&body).map(|x| x.0.to_string().into()).unwrap_or(body.trim_start_matches("uBody_").into()) };
                        let item = CustomAssetMenuItem::new_asset(AssetType::Body, data.hash, name, current == data.hash, preview.original_assets[0] == data.hash);
                        this.full_menu_item_list.add(item);
                    }
                });
            }
            ShopBody((2, alt))  => {
                let current = UnitAssetMenuData::get_current_unit_hash(AssetType::Body);
                let female = UnitAssetMenuData::get_gender(*alt) == 2;
                let db = get_outfit_data();
                if female { &db.list.engaged.female } else { &db.list.engaged.male}.iter().for_each(|x| {
                    let name = x.label.get();
                    let item = CustomAssetMenuItem::new_asset(AssetType::Body, x.hash, name, current == x.hash, preview.original_assets[0] == x.hash);
                    this.full_menu_item_list.add(item);
                });
            }
            ShopBody((3, alt)) => {
                let current = UnitAssetMenuData::get_current_unit_hash(AssetType::Body);
                let female = UnitAssetMenuData::get_gender(*alt) == 2;
                let db = get_outfit_data();
                if female { &db.list.other_outfits.female } else { &db.list.other_outfits.male}.iter().for_each(|x| {
                    let mut name = x.label.get();
                    if name.to_string().len() < 2 {
                        name = db.try_get_asset(AssetType::Body, x.hash).map(|v| v.to_string().into()).unwrap_or("Unknown".into());
                    }
                    let item = CustomAssetMenuItem::new_asset(AssetType::Body, x.hash, name, current == x.hash, preview.original_assets[0] == x.hash);
                    this.full_menu_item_list.add(item);
                });
            }
            Head => {
                let current = UnitAssetMenuData::get_current_unit_hash(AssetType::Head);
                let photo = UnitAssetMenuData::is_photo_graph();
                db.list.head_list.iter().filter(|x| (photo && x.label.name.contains("Morph")) || !photo)
                    .for_each(|x| {
                        let name = if x.get_name().to_string().len() < 3 {
                            if let Some(head) = db.try_get_asset(AssetType::Head, x.hash).map(|v| v.to_string()) { head.split("_c").last().unwrap().into() }
                            else { Mess::get("MPID_Unknown") }
                        }
                        else { x.get_name() };
                        this.full_menu_item_list
                            .add(CustomAssetMenuItem::new_asset(AssetType::Head, x.hash, name, current == x.hash, preview.original_assets[1] == x.hash));
                });
            }
            Hair => {
                let current = UnitAssetMenuData::get_current_unit_hash(AssetType::Hair);
                let original = preview.original_assets[2];
                db.list.hair_list.iter().for_each(|h|{
                    let name =
                        if let Some(hair) = db.try_get_asset(AssetType::Hair, h.hash).map(|v| v.to_string()) {
                            let s = h.get_name();
                            if s.to_string().len() < 2 { hair.trim_start_matches("uHair_").into() }
                            else { s }
                        }
                        else { Mess::get("MPID_Unknown") };
                    let item = CustomAssetMenuItem::new_asset(AssetType::Hair, h.hash, name, current == h.hash, preview.original_assets[2] == h.hash);
                    item.padding = (h.hash == original) as i32;
                    this.full_menu_item_list.add(item);
                });

            }
            Rig => {
                let current = UnitAssetMenuData::get_current_unit_hash(AssetType::Rig);
                let original = preview.original_assets[15];
                db.hashes.rigs.iter().for_each(|(h, n)|{
                    let name = n.trim_start_matches("uRig_").into();
                    let item = CustomAssetMenuItem::new_asset(AssetType::Rig, *h, name, current == *h, original == *h);
                    this.full_menu_item_list.add(item);
                });
            }
            ShopAcc(kind) => {
                let index = *kind as usize;
                let set = if index < 2 { &db.list.acc[0] } else { &db.list.acc[index - 1] };
                let current = UnitAssetMenuData::get_current_unit_hash(AssetType::Acc(*kind));
                let original = preview.original_assets[*kind as usize + 5];
                set.iter().enumerate().for_each(|(i, x)|{
                    let name =
                        if i == 0 { Mess::get("MID_SYS_None") }
                        else if index == 3 { db.try_get_asset(AssetType::Acc(*kind), x.hash).unwrap().to_string().trim_start_matches("uAcc_Eff_").into() }
                        else { x.get_name(db) };
                    this.full_menu_item_list.add(CustomAssetMenuItem::new_asset(AssetType::Acc(*kind), x.hash, name, current == x.hash, original == x.hash));
                });
            }
            ShopAoc(page) => {
                let current = UnitAssetMenuData::get_current_unit_hash(AssetType::AOC(*page));
                let db = get_outfit_data();
                let gender = db.get_dress_gender_hash(UnitAssetMenuData::get_current_unit_hash(AssetType::Body)).map(|g|
                    if g == Gender::Male { 1 } else { 2 }
                ).unwrap_or(UnitAssetMenuData::get_current_dress_gender());
                let set = if gender == 2 { &db.list.aoc_info_f[*page as usize] } else { &db.list.aoc_info_m[*page as usize] };
                set.iter().for_each(|x| {
                    let name =
                        if let Some(aoc) = db.try_get_asset(AssetType::AOC(0), *x).map(|v| v.to_string()) {
                            let engaged = aoc.contains("_Eng");
                            let aoc = if engaged { aoc.trim_end_matches("_Eng") } else { aoc.as_str() };
                            db.labels.asset.get(x).map(|x| x.get())
                                .or_else(||db.labels.suffixes.iter().find(|x| aoc.ends_with(*x.0))
                                    .map(|x| if engaged { format!("{} {}", x.1.get(), MenuTextCommand::Engage).into() } else { x.1.get() })
                                ).or_else(||
                                    db.labels.suffixes.iter().find(|x| aoc.contains(*x.0))
                                        .map(|x|
                                            if engaged { format!("{} {}", x.1.get(), MenuTextCommand::Engage) }
                                            else { format!("{} {}", x.1.get(), aoc.split(x.0).last().unwrap_or("")) }.into()
                                        )
                                ).unwrap_or(aoc.split("_").last().unwrap().into())
                        } else { Mess::get("MPID_Unknown") };
                    this.full_menu_item_list.add(CustomAssetMenuItem::new_asset(AssetType::AOC(*page), *x, name, current == *x, preview.original_assets[10+*page as usize] == *x));
                });
            }
            ShopMount(mount) => {
                let index = *mount as i32;
                let set = &db.list.mount[index as usize];
                let current_mount = UnitAssetMenuData::get_current_unit_hash(AssetType::Mount(*mount));
                set.iter().for_each(|hash|{
                    if let Some(m) = db.hashes.mounts.get(hash).map(|v| v.to_string()) {
                        let name = if m.contains("Box") { "Box".into() }
                        else if let Some(s1) = db.try_get_suffix(&m).filter(|p| !p.1.contains("000")) {
                            if let Some(s2) = db.try_get_body_label(&m) { format!("{} ({})", s2.0, s1.0).into() }
                            else { s1.0 }
                        }
                        else if let Some(s) = db.try_get_body_label(&m) { s.0 }
                        else if let Some(s1) = db.try_get_suffix(&m) {
                            if let Some(s2) = db.try_get_body_label(&m) { format!("{} ({})", s2.0, s1.0).into() }
                            else { s1.0 }
                        }
                        else { m.trim_start_matches("uBody_").into() };
                        this.full_menu_item_list.add(CustomAssetMenuItem::new_asset(AssetType::Mount(*mount), *hash, name, current_mount == *hash, false));
                    }
                });
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
                let current = UnitAssetMenuData::get_current_unit_hash(AssetType::Voice);
                db.list.voice.iter().for_each(|h|{
                    let name = h.get_name();
                    this.full_menu_item_list.add(CustomAssetMenuItem::new_asset(AssetType::Voice, h.hash, name, current == h.hash, preview.original_assets[14] == h.hash));
                });
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
                if UnitAssetMenuData::get_gender(*alt) == 2 { &db.list.class_female }
                else { &db.list.class_male }.get(*class as usize).map(|v| Mess::get(v.class_label.as_str())).unwrap()
            }
            ShopAcc(_) => { Mess::get("MID_Hub_Mascot_Accessories_Parts") }
            VoiceSelection => { MenuTextCommand::Voice.get() }
            Head => { Mess::get("MID_Hub_Mascot_Accessories_Head") }
            Personal => { MenuTextCommand::Personal.get() }
            _ => { self.get_menu_item_name().unwrap() }
            /*
            ProfileSelection => { MenuText::get_command(37) }
            Hair => { MenuText::get_command(16) }
            ShopMount(_) => { MenuText::get_command(25) }
            ShopAoc(_) => { MenuText::get_command(24) }
            ScaleMenu => { MenuText::get_command(36) }
            ColorKindSelection => { MenuText::get_command(35) }
            ColorSelection(kind) => { MenuText::get_command(70+*kind as i32) }
            RGBAMenu(_) => { MenuText::get_command(39) }
            Rig => { MenuText::get_command(62)  }
            PresetAppearanceMenu(_) => { MenuText::get_command(63) }


            _ => { "None".into() }
             */
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