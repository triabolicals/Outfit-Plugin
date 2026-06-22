use bitflags::{bitflags, Flags};
use engage::{
    app::{IAccessoryDataMethods, IAssetTable_ConditionFlags, IBitField32, IGameUserDataMethods, IGodDataMethods, IGodUnit, IItemDataMethods, IPersonDataMethods, ISingletonClass_1Methods, ISkillArrayMethods, IStructBase, IStructData_1Methods, IUnit, IUnitAccessory, IUnitAccessoryList, IUnitEdit, IUnitMethods},
    List_1Ext
};
use unity::Cast;
use crate::{get_condition_index, get_outfit_data, UnitAssetMenuData};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum RandomDressMode {
    Off,
    Static,
    Random,
    Chaos,
}
impl RandomDressMode {
    pub fn is_off(&self) -> bool {
        *self == RandomDressMode::Off
    }
    pub fn new() -> Self {
        if UnitAssetMenuData::get().is_dvc {
            let s = engage::GameVariableManager::get_number("G_RandomJobOutfit") & 255;
            match s {
                1 => RandomDressMode::Static,
                2 => RandomDressMode::Random,
                3 => RandomDressMode::Chaos,
                _ => RandomDressMode::Off,
            }
        }
        else { RandomDressMode::Off }
    }
    pub fn get_random(&self, unit: engage::app::Unit, base_seed: i32) -> Option<engage::app::Random_2> {
        let god =
            if unit.is_engaging_2() {
                let god = unit.get_god_unit();
                if !god.is_null() { god.m_data().hash() }
                else { 0 }
            }
            else { 0 };

        let job = if engage::app::GameUserData::get_instance().get_sequence().value  == 4 { 0 } else { (unit.get_job().hash() >> 2) +  unit.m_selected_weapon_mask().m_value() };
        match self {
            RandomDressMode::Off => { None }
            RandomDressMode::Static => {
                let seed = (base_seed >> 1) + (unit.get_person().hash() >> 1) + job +  god;
                Some(engage::app::Random_2::new_2(seed as u32))
            }
            RandomDressMode::Random => {
                let seed = (base_seed >> 1) + (unit.get_person().hash() >> 1) + job + (unit.m_grow_seed() as i32  >> 2) + god;
                Some(engage::app::Random_2::new_2(seed as u32))
            }
            RandomDressMode::Chaos => { Some(engage::app::Random_2::get_system()) }
        }
    }
}
pub struct AssetConditions {
    pub character_mode: CharacterAssetMode,
    pub flags: AssetFlags,
    pub engaged: Option<String>,
    pub kind: i32,
    pub mode: i32,
    pub emblem_unit: bool,
    pub broken: bool,
    pub profile_flag: i32,
    pub random_dress: RandomDressMode,
}
impl AssetConditions {
    pub fn new(unit: engage::app::Unit, mode: i32, item: engage::app::ItemData) -> Self {
        let engaged =
            if !unit.is_null() && unit.is_engaging_2() {
                let god_unit = unit.get_god_unit();
                if !god_unit.is_null() {
                    Some(god_unit.m_data().get_gid().to_rust_string())
                }
                else { None }
            }
            else { None };
        let kind =
            if !item.is_null() {
                if item.get_flag().m_value() & 67108864 != 0 { 9 }
                else if item.get_flag().m_value() & 134217728 != 0 { 10 }
                else if item.get_kind().value < 9 { item.get_kind().value }
                else { 0 }
            }
            else { 0 };
        let broken = if !unit.is_null() { unit.m_private_skill().find("SID_気絶").is_null() || unit.m_mask_skill().find("SID_気絶").is_null() } else { false };

        Self {
            kind, mode, broken, engaged,
            profile_flag: 0, emblem_unit: false,
            random_dress: RandomDressMode::new(),
            character_mode: CharacterAssetMode::get(),
            flags: AssetFlags::new(unit),
        }
    }
    pub fn remove_god_eid_conditions(&mut self) {
        if !self.flags.contains(AssetFlags::EngageTiki){
            if let Some(eid) = self.engaged.as_ref() {
                AssetFlags::set_condition_key(eid.as_str(), false);
                AssetFlags::set_condition_key(eid.replace("GID_", "EID_"), false);
            }
        }
    }

}
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CharacterAssetMode {
    ClassChange,    // クラスチェンジ中
    Detail, //  詳細
    UnitInfo,   //  情報
    Hub,    //  拠点
    Demo,   //  デモ
    Talk,   //  会話
    Combat, // コンバット
    PrivateClothes, // 私服
    None,
}
impl CharacterAssetMode {
    const CONDITIONS: [&'static str; 8] = ["クラスチェンジ中", "詳細", "情報", "拠点", "デモ", "会話", "コンバット", "私服"];
    pub fn get() -> Self {
        let flags = engage::app::AssetTable::s_condition_flags();
        if let Some(pos) = Self::CONDITIONS.iter().position(|x| flags.m_keys().iter().any(|s| s.to_rust_string().contains(x))){
            match pos {
                0 => CharacterAssetMode::ClassChange,
                1 => CharacterAssetMode::Detail,
                2 => CharacterAssetMode::UnitInfo,
                3 => CharacterAssetMode::Hub,
                4 => CharacterAssetMode::Demo,
                5 => CharacterAssetMode::Talk,
                6 => CharacterAssetMode::Combat,
                7 => CharacterAssetMode::PrivateClothes,
                _ => CharacterAssetMode::None,
            }
        }
        else { CharacterAssetMode::None }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct AssetFlags: i32 {
        const CausalClothes = 1 << 0;
        const Corrupted = 1 << 1;
        const GenericSoldier = 1 << 2;
        const Phantom = 1 << 3;
        const Vision = 1 << 4;
        const Engaging = 1 << 5;
        const Engaged = 1 << 6;
        const EngAtkHUP = 1 << 7;
        const EngAtkCoop = 1 << 8;
        const EngAtkCoopMain = 1 << 9;
        const EngAtkCoopSub = 1 << 10;
        const EngageTiki = 1 << 11;
        const DragonStone = 1 << 12;
        const MapTransform = 1 << 13;
        const Dance = 1 << 14;
        const Ballista = 1 << 15;
        const Bullet = 1 << 16;
        const Male = 1 << 17;
        const Female = 1 << 18;
        const Emblem = 1 << 19;
        const DarkEmblem = 1 << 20;
        const ClassChange = 1 << 21;
        const ThreeStar = 1 << 22;
        const FiveStar = 1 << 23;
        const EngageAttack = 1 << 24;
        const AxeStaff = 1 << 25;
        // Derived Flags
        const HumanTikiEngage = 1 << 26;
        const Summon = 1 << 27;
        const CombatTranforming = 1 << 28;
        const NeedCommonClass = 1 << 29;
        const NPC = 1 << 30;
        const Monster = 1 << 31;
    }
}

impl AssetFlags {
    pub const UNIT_STATUS_ENGAGED: u64 = 8388608;
    pub const UNIT_STATUS_ENGAGE_ATK: u64 = 16777216;
    pub const UNIT_STATUS_ENGAGE_LINK: u64 = 33554432;
    pub const UNIT_STATUS_VISION: u64 = 134217728;
    pub const UNIT_STATUS_SUMMON: u64 = 35184372088832;
    pub const ASSET_TABLE_CONDITIONS: [&'static str; 26] = [
        "私服", "AID_異形兵", "AID_一般兵", "AID_幻影兵", "残像",
        "エンゲージ開始", "エンゲージ中", "協力エンゲージ技", "エンゲージ合体技",
        "メイン", "サブ", "EID_チキ",
        "竜石", "竜化", "踊り", "砲台", "弾丸",
        "男装", "女装", "神将", "闇化", "クラスチェンジ中", "☆3", "☆5", "エンゲージ技",
        "AID_ヴェロニカ_フリズスキャルヴ",
    ];
    pub fn new(unit: engage::app::Unit) -> Self {
        let flags = engage::app::AssetTable::s_condition_flags();

        let bits =
            Self::ASSET_TABLE_CONDITIONS.iter()
            .enumerate()
            .filter_map(|(i, con)| Some(i).zip(get_condition_index(*con)))
            .filter(|(i, idx)| flags.m_bits().get(*idx))
            .fold(0, |x, (i, idx)| x | (1 << i));
        let mut flags = Self::from_bits(bits).unwrap();
        if !unit.is_null() {
            // if UnitAssetMenuData::is_photo_graph() { unit.accessory_list.clear(); }
            let vision = unit.is_vision();
            let condition_unit = if vision { engage::app::UnitUtil::get_vision_owner(unit) } else { unit };
            if condition_unit.get_person().get_bmap_size() > 1 || condition_unit.get_gender().value & 3 == 0 {
                // Unit is greater than 1x1 or does not have a gender
                flags.set(AssetFlags::Monster, true);
                return flags;
            }
            let mut transform_tiki = condition_unit.get_pid().to_rust_string().contains("G001_チキ_");
            let person = condition_unit.get_person();
            let aid = person.get_aid();
            if !aid.is_null() { transform_tiki |= aid.to_rust_string().contains("竜化"); }
            if transform_tiki {
                flags.set(AssetFlags::Monster, true);
                return flags;
            }
            if condition_unit.is_summon() { flags.set(AssetFlags::Summon, true); }
            if get_outfit_data().is_monster_class(unit) {
                Self::set_condition_key(unit.get_jid(), false);
                flags.set(AssetFlags::CombatTranforming, true);
                flags.set(AssetFlags::Monster, false);
            }
            if person.get_flag().m_value() & 32 != 0 {
                let gender = if unit.get_person().get_gender().value == 2 { engage::app::Gender::male() } else { engage::app::Gender::female() };
                flags.set_gender(gender);
            }
            else { flags.set_gender(unit.get_person().get_gender()); }
            if person.index() == 1 || person.get_flag().m_value() & 128 != 0 {
                let edit_gender = condition_unit.m_edit().m_gender();
                if edit_gender.value & 3 != 0 { flags.set_gender(edit_gender); }
            }
            if person.get_name().to_rust_string().contains("Boss") { flags.set(AssetFlags::NPC, true); }
            if flags.contains(AssetFlags::EngageTiki) {
                let tiki = engage::app::GodData::get("GID_チキ".into());
                if tiki.get_flag().m_value() & 16 == 0 {
                    flags.set(AssetFlags::HumanTikiEngage, true);
                    flags.set_condition_flag(AssetFlags::EngageTiki, false);
                }
            }
        }
        flags
    }
    pub fn set_gender(&mut self, gender: engage::app::Gender) {
        self.set_condition_flag(AssetFlags::Male, gender == engage::app::Gender::male());
        self.set_condition_flag(AssetFlags::Female, gender == engage::app::Gender::none());
    }
    pub fn set_condition_flag(&mut self, rhs: Self, value: bool){
        if let Some(condition) = Self::FLAGS.iter()
            .position(|p| p.value().bits() == rhs.bits())
            .filter(|&p| p < Self::ASSET_TABLE_CONDITIONS.len())
            .and_then(|pos| Self::ASSET_TABLE_CONDITIONS.get(pos))
        {
            Self::set_condition_key(*condition, value);
        }
        self.set(rhs, value);
    }
    pub fn get_condition_index(key: impl Into<unity::Il2CppString>) -> Option<i32> { get_condition_index(key) }
    pub fn set_condition_key(key: impl Into<unity::Il2CppString>, value: bool){
        if let Some(index) = get_condition_index(key) {
            engage::app::AssetTable::s_condition_flags().m_bits().set(index, value);
        }
    }
    pub fn remove_accessory_conditions(acc: engage::app::UnitAccessory) {
        let index = acc.m_index();
        if index < 1 { return; }
        let acc = engage::app::AccessoryData::try_get_2(index);
        if !acc.is_null() {
            Self::set_condition_key(acc.get_aid(), false);
            let asset = acc.get_asset();
            if !asset.is_null() {
                Self::set_condition_key(asset, false);
            }
        }
    }
    pub fn set_person_conditions(person: engage::app::PersonData, value: bool) {
        if person.is_null() { return; }
        Self::set_condition_key(person.get_pid(), value);
        let name = person.get_name();
        if !name.is_null() { Self::set_condition_key(name, value); }
        let aid = person.get_aid();
        if !aid.is_null() { Self::set_condition_key(aid, value); }
        let bid = person.get_belong();
        if !bid.is_null() { Self::set_condition_key(bid, value); }
    }
    pub fn remove_unit_accessories(unit: engage::app::Unit) {
        if !unit.is_null() {
            let accessory_list = unit.get_accessory_list();
            accessory_list.m_unit_accessorys().iter().for_each(|a| { Self::remove_accessory_conditions(a); });
        }
    }
    pub fn remove_mount(&self) -> bool { self.bits() & 131040 != 0 }
    pub fn is_generic(&self) -> bool {
        !self.contains(Self::NPC) && (self.contains(Self::GenericSoldier) || self.contains(Self::Corrupted) || self.contains(Self::Phantom))
    }
}