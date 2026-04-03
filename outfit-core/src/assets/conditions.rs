use bitflags::{bitflags, Flags};
use engage::gamedata::assettable::AssetTableStaticFields;
use engage::gamedata::{Gamedata, GodData, PersonData};
use engage::gamedata::accessory::AccessoryData;
use engage::unit::{Gender, Unit, UnitAccessory, UnitStatusField, UnitUtil};
use unity::system::Il2CppString;
use engage::gamedata::item::ItemData;
use engage::gameuserdata::GameUserData;
use engage::gamevariable::GameVariableManager;
use engage::random::Random;
use crate::{get_outfit_data, UnitAssetMenuData};

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
            let s = GameVariableManager::get_number("G_RandomJobOutfit") & 255;
            match s {
                1 => RandomDressMode::Static,
                2 => RandomDressMode::Random,
                3 => RandomDressMode::Chaos,
                _ => RandomDressMode::Off,
            }
        }
        else {
            RandomDressMode::Off
        }
    }
    pub fn get_random(&self, unit: &Unit, base_seed: i32) -> Option<&'static Random> {
        let god =
            if unit.status.value & UnitStatusField::Engaging != 0 { unit.god_link.or(unit.god_unit)
                .map(|v| v.data.parent.hash >> 2)
                .unwrap_or(0) }
            else { 0 };

        let job = if GameUserData::get_sequence() == 4 { 0 } else { (unit.job.parent.hash >> 2) +  unit.selected_weapon_mask.value };
        match self {
            RandomDressMode::Off => { None }
            RandomDressMode::Static => {
                let seed = (base_seed >> 1) + (unit.person.parent.hash >> 1) + job +  god;
                Some(Random::new(seed as u32))
            }
            RandomDressMode::Random => {
                let seed = (base_seed >> 1) + (unit.person.parent.hash >> 1) + job + (unit.grow_seed >> 2) + god;
                Some(Random::new(seed as u32))
            }
            RandomDressMode::Chaos => {
                Some(Random::get_system())
            }
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
    pub fn new(unit: Option<&Unit>, mode: i32, item: Option<&ItemData>) -> Self {
        let engaged = 
            unit.as_ref()
                .filter(|x| x.status.value & UnitStatusField::Engaging != 0 )
                .and_then(|u| u.god_link.or(u.god_unit))
                .map(|g_unit| g_unit.data.asset_id.to_string());

        let kind = item.map(|x|{
            if x.flag.value & 67108864 != 0 { 9 }
            else if x.flag.value & 134217728 != 0 { 10 }
            else if x.kind < 9 { x.kind as i32 }
            else { 0 }
        }).unwrap_or(0);
        let broken = unit.as_ref()
            .map(|v| v.private_skill.find_sid("SID_気絶").is_some() || v.mask_skill.is_some_and(|v| v.find_sid("SID_気絶").is_some())).unwrap_or(false);

        Self {
            broken,
            profile_flag: 0,
            engaged, character_mode: CharacterAssetMode::get(), flags: AssetFlags::new(unit), kind, mode, emblem_unit: false, random_dress: RandomDressMode::new() }
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
    None,
}
impl CharacterAssetMode {
    const CONDITIONS: [&'static str; 7] = ["クラスチェンジ中", "詳細", "情報", "拠点", "デモ", "会話", "コンバット"];
    pub fn get() -> Self {
        let sf = AssetTableStaticFields::get();
        if let Some(pos) = Self::CONDITIONS.iter().position(|x|{ sf.condition_flags.keys.iter().find(|x2| x2.str_contains(x)).is_some() }) {
            match pos {
                0 => CharacterAssetMode::ClassChange,
                1 => CharacterAssetMode::Detail,
                2 => CharacterAssetMode::UnitInfo,
                3 => CharacterAssetMode::Hub,
                4 => CharacterAssetMode::Demo,
                5 => CharacterAssetMode::Talk,
                6 => CharacterAssetMode::Combat,
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
    pub fn new(unit: Option<&Unit>) -> Self {
        let sf = AssetTableStaticFields::get();
        let mut bits = 0;
        Self::ASSET_TABLE_CONDITIONS.iter()
            .enumerate()
            .for_each(|(i, con)| {
                let index = AssetTableStaticFields::get_condition_index(con);
                if index >= 0 || index < sf.condition_flags.bits.bits.len() as i32 * 4 { if sf.condition_flags.bits.get(index) { bits |= 1 << i; } }
            });

        let mut flags = Self::from_bits(bits).unwrap();
        if let Some(unit) = unit {
            if UnitAssetMenuData::is_photo_graph() { unit.accessory_list.clear(); }
            let vision = unit.status.value & UnitStatusField::Vision != 0;
            let condition_unit = if vision { UnitUtil::get_vision_owner(unit).unwrap_or(unit) } else { &unit };
            if condition_unit.person.bmap_size > 1 || condition_unit.person.get_gender() & 3 == 0 {
                // Unit is greater than 1x1 or does not have a gender
                flags.set(AssetFlags::Monster, true);
                return flags;
            }
            if unit.person.aid.is_some_and(|x| x.to_string().contains("竜化")) {
                flags.set(AssetFlags::Monster, true);
                return flags;
            }
            if condition_unit.status.value & UnitStatusField::Summon != 0 { flags.set(AssetFlags::Summon, true); }
            if get_outfit_data().is_monster_class(unit) {
                println!("{} is in a monster class: #{} {}", unit.get_name(), unit.job.parent.index, engage::mess::Mess::get_name(unit.job.jid));
                Self::set_condition_key(unit.job.jid, false);
                flags.set(AssetFlags::CombatTranforming, true);
                flags.set(AssetFlags::Monster, false);
            }
            if unit.person.flag.value & 32 != 0 {
                let gender = if unit.person.get_gender() == 2 { Gender::Male } else { Gender::Female };
                flags.set_gender(gender);
            } else {
                flags.set_gender(unit.person.get_gender2());
            }
            if unit.person.parent.index == 1 || unit.person.flag.value & 128 != 0 {
                if unit.edit.gender & 3 != 0 {
                    flags.set_gender(if unit.edit.gender == 1 { Gender::Male } else { Gender::Female });
                }
            }
            if unit.person.name.is_some_and(|x| x.str_contains("Boss")) { flags.set(AssetFlags::NPC, true); }
            if flags.contains(AssetFlags::EngageTiki) {
                if GodData::get("GID_チキ").map(|g| g.flag.value & 16 == 0).unwrap_or(false) {
                    flags.set(AssetFlags::HumanTikiEngage, true);
                    flags.set_condition_flag(AssetFlags::EngageTiki, false);
                }
            }
        }
        flags
    }
    pub fn set_gender(&mut self, gender: Gender) {
        self.set_condition_flag(AssetFlags::Male, gender == Gender::Male);
        self.set_condition_flag(AssetFlags::Female, gender == Gender::Female);
    }
    pub fn set_condition_flag(&mut self, rhs: Self, value: bool){
        if let Some(condition) = Self::FLAGS.iter()
            .position(|p| p.value().bits() == rhs.bits())
            .filter(|&p| p < Self::ASSET_TABLE_CONDITIONS.len())
            .and_then(|pos| Self::ASSET_TABLE_CONDITIONS.get(pos))
        {
            Self::set_condition_key(condition, value);
        }
        self.set(rhs, value);
    }
    pub fn set_condition_key(key: impl Into<&'static Il2CppString>, value: bool){
        let index = AssetTableStaticFields::get_condition_index(key);
        let sf = AssetTableStaticFields::get();
        if index >= 0 {
            sf.condition_flags.bits.set(index, value);
        }

    }
    pub fn remove_accessory_conditions(acc: &UnitAccessory) {
        if acc.index < 1 { return; }
        if let Some(acc) = AccessoryData::try_index_get_mut(acc.index) {
            let sf = AssetTableStaticFields::get();
            let index = AssetTableStaticFields::get_condition_index(acc.aid);
            if index > 0 { sf.condition_flags.bits.set(index, false); }
            if !acc.asset.is_null() {
                let index2 = AssetTableStaticFields::get_condition_index(acc.asset);
                if index2 > 0 { sf.condition_flags.bits.set(index2, false); }
            }
        }
    }
    pub fn set_person_conditions(person: &PersonData, value: bool) {
        Self::set_condition_key(person.pid, value);
        if let Some(name) = person.name { Self::set_condition_key(name, value); }
        if let Some(aid) = person.aid { Self::set_condition_key(aid, value); }
        if let Some(bid) = person.belong { Self::set_condition_key(bid, value); }
    }
    pub fn remove_unit_accessories(unit: &Unit) {
        unit.accessory_list.unit_accessory_array.iter().for_each(|a| {
            Self::remove_accessory_conditions(a);
        });
    }
    pub fn remove_mount(&self) -> bool { self.bits() & 131040 != 0 }
    pub fn is_generic(&self) -> bool {
        !self.contains(Self::NPC) && (self.contains(Self::GenericSoldier) || self.contains(Self::Corrupted) || self.contains(Self::Phantom))
    }
}