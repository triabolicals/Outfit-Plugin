use engage_il2cpp::app::{AccessoryData_Kinds, GameIcon, ISpriteAtlasManager_2};
use engage_il2cpp::system::collections::generic::IDictionary_2Methods;
use unity2::Cast;

#[repr(u8)]
#[derive(PartialEq, Copy, Clone)]
pub enum CustomMenuIcon {
    Clothes,
    Body,
    Head,
    Hair,
    Talk,
    AccFace,
    Shield,
    StarBlank,
    Star,
    EngageCommon,
    Engaged(u8),
    Rare,
    KeyItem,
    TimeCrystal,
    RedEngWep,
    GreenEngWep,
    BlueEngWep,
    WhiteEngWep,
    Mount(u8),
    Armor,
    Gift,
    Horse,
    Dragon,
    Fly,
    Weapon,
    SolaTail,
    Satchel,
    GiftCategory,
    UnitAccessory{kind: u8},
    TalkStory,
    Day,
    SilverCard,
    NoIcon,
    Color,

}
impl CustomMenuIcon {
    pub fn get_system_label(&self) -> Option<&'static str> {
        match self {
            Self::Clothes => Some("Clothes"),
            Self::Body => Some("Body"),
            Self::Head => Some("SolaHeadCategory"),
            Self::Hair => Some("SolaHead"),
            Self::Talk => Some("TalkRelianceOutline"),
            Self::AccFace => Some("Face"),
            Self::KeyItem => Some("KeyItem"),
            Self::TimeCrystal => Some("TimeCrystal"),
            Self::EngageCommon => Some("EngCommon"),
            Self::Gift => Some("Gift"),
            Self::GiftCategory => Some("GiftCategory"),
            Self::Rare => Some("Rare"),
            Self::StarBlank => Some("StarBlank_Small"),
            Self::Star => Some("Star_Small"),
            Self::SolaTail => Some("SolaTail"),
            Self::RedEngWep => Some("EngWepAtk"),
            Self::GreenEngWep => Some("EngWepSpd"),
            Self::BlueEngWep => Some("EngWepDef"),
            Self::WhiteEngWep => Some("EngWepSkill"),
            Self::Satchel => Some("EnchantSeal"),
            Self::TalkStory => Some("TalkStoryOutline"),
            Self::Weapon => Some("Weapon"),
            Self::Day => Some("Day"),
            Self::SilverCard => Some("SilverCard"),
            _ => None,
        }
    }
    pub fn get_icon(&self) -> Option<engage_il2cpp::unity_engine::Sprite> {
        if let Some(system_label) = self.get_system_label() { to_option_sprite(GameIcon::try_get_system(system_label)) } else {
            match self {
                Self::Engaged(i) => {
                    let idx = if *i < 13 { *i as usize } else { *i as usize + 1 };  // Skip Tiki
                    if *i < 19 {
                        let (found, sprite) = GameIcon::s_god_symbol().m_cache_table().try_get_value(crate::EMBLEM[idx].0.into());
                        if found { Some(sprite) } else { None }
                    } else { to_option_sprite(GameIcon::try_get_system("EngCommon")) }
                }
                Self::UnitAccessory { kind } => { to_option_sprite(GameIcon::try_get_accessory_kinds(AccessoryData_Kinds{value: *kind as i32})) }
                Self::Shield => { to_option_sprite(GameIcon::try_get_item("Byleth_AegisShield")) }
                Self::Mount(i) => {
                    let icon_label = match i { 2 => "Dragon", 3 | 4 => "Fly", _ => "Horse"};
                    to_option_sprite(GameIcon::try_get_efficacy(icon_label, true))
                }
                Self::Armor => { to_option_sprite(GameIcon::try_get_efficacy("Armor", true)) }
                Self::Horse => { to_option_sprite(GameIcon::try_get_efficacy("Horse", true)) }
                Self::Dragon => { to_option_sprite(GameIcon::try_get_efficacy("Dragon", true)) }
                Self::Fly => { to_option_sprite(GameIcon::try_get_efficacy("Fly", true)) }
                _ => None,
            }
        }
    }
}
fn to_option_sprite(s: engage_il2cpp::unity_engine::Sprite) -> Option<engage_il2cpp::unity_engine::Sprite> { if s.is_null() { None } else { Some(s) } }