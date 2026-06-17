use std::sync::OnceLock;
use engage::{
    menu::{menu_item::{MenuItem, MenuItemContent}, BasicMenuResult},
    menu::BasicMenuMethods
};
use engage_il2cpp::{
    app::{ISingletonClass_1Methods, accessoryequipmentinfo::*, IHubAccessoryRoomMethods, AssetTable_Modes, AssetTable_Result, IBasicMenuItemMethods, IGodDataMethods, IPersonDataMethods, ISingletonProcInst_1Methods, IStructBase, IStructData_1Methods, IUnit, IUnitEdit, IUnitMethods, IAssetTable_ResultMethods, ShopUnitSelectMenuItem, Proc, IProcInstMethods, IHubAccessoryShopSequence, AccessoryShopTopMenu_Result2, BasicMenu_Result, IAccessoryShopUnitSelectRoot, IHubAccessoryShopSequenceMethods, IShopUnitSelectMenuItemContent, IUnitMenuItemSetter, IShopUnitSelectMenuItemContentMethods, ShopUnitSelectMenuItemContent, ShopUnitSelectMenu, IBasicMenu, IBasicMenuMethods, ISingletonPool_2, IGodUnit, IGodUnitMethods, IGameUserDataMethods, BasicMenuItem},
    List_1Ext,
    system::collections::generic::IList_1Methods,
    tm_pro::ITMP_Text,
    unity_engine::{IComponentMethods, IGameObjectMethods, ui::IImageMethods}
};
use unity2::{Cast, Class, FromIlInstance, IlNull};
use crate::{EquipmentBoxMode, EquipmentBoxPage, UnitAssetMenuData, room::ReloadType, shop::room::hub_room_set_by_result, CustomAssetMenu, get_default_asset_conditions};

#[derive(Default)]
pub struct UnitSelectList {
    pub selected: Option<i32>,
    pub list: Vec<UnitSelect>,
}
impl UnitSelectList {
    pub const fn new() -> Self { Self { selected: None, list: Vec::new(), } }
    pub fn init(&mut self) {
        self.selected = Some(0);
        self.list.clear();
        let data = &UnitAssetMenuData::get().data;
        let unit = engage_il2cpp::app::UnitPool::get_hero(false);
        if !unit.is_null() {
            self.list.push(UnitSelect::from_unit(unit));
            let mut start = unit;
            loop {
                let u = engage_il2cpp::app::UnitFor::get_next_2(start, 9u32);
                if !u.is_null() {
                    if !self.list.iter().any(|v| v.hash == u.get_person().hash()) {
                        self.list.push(UnitSelect::from_unit(u));
                    }
                    start = u;
                } else { break; }
            }
        }
        data.iter().for_each(|data| {
            if !self.list.iter().any(|v| v.hash == data.person) {
                let person = engage_il2cpp::app::PersonData::try_get_from_hash(data.person);
                if !person.is_null() { self.list.push(UnitSelect::from_person(person)); }
            }
        });
        let god_pool = engage_il2cpp::app::GodPool::get_instance();
        if !god_pool.is_null() {
            god_pool.m_sort().iter()
                .filter(|g_unit| !g_unit.m_is_escaping() && g_unit.get_force_type().value == 0)
                .for_each(|g_unit| {
                    g_unit.m_data().get_change_data().iter().for_each(|god|{
                        if !self.list.iter().any(|v| v.hash == god.hash()) { self.list.push(UnitSelect::from_god(god)); }
                    });
                });
        }
    }
    pub fn change(&mut self, next: bool) {
        if let Some(current) = self.selected.as_ref() {
            let size = self.list.len() as i32;
            let new = if next { (*current + 1) % size } else { (*current + size - 1) % size };
            self.selected = Some(new);
        }
        else { self.selected = Some(0); }
        UnitAssetMenuData::get().unit_select_index = self.selected.unwrap_or(0);
    }
    pub fn get_selected(&self) -> Option<UnitSelect> { self.selected.and_then(|v| self.list.get(v as usize).cloned()) }
    pub fn get_result(&self, hub: bool) -> AssetTable_Result {
        if let Some(result) = self.get_selected().map(|v| v.get_result(hub)) { result }
        else {
            AssetTable_Result::get_from_pid(
                AssetTable_Modes::combat(),
                "PID_リュール",
                engage_il2cpp::combat::CharacterAppearance::get_constions(get_default_asset_conditions())
            )
        }

    }
}
#[derive(Default, Clone, Copy)]
pub struct UnitSelect{
    pub hash: i32,
    pub god: bool,
    pub recruited: bool,
    pub female: bool,
}
impl UnitSelect {
    pub fn from_person(person: engage_il2cpp::app::PersonData) -> Self {
        let female = person.get_dress_gender().value == 2;
        Self{ hash: person.hash(), god: false, recruited: false, female, }
    }
    pub fn from_god(god: engage_il2cpp::app::GodData) -> Self {
        let female =
            if god.is_hero() {
                let unit = engage_il2cpp::app::UnitPool::get_hero(false);
                if unit.is_null() { false } else { unit.m_edit().m_gender() == engage_il2cpp::app::Gender::female() }
            }
            else { god.get_female() != 0 };
        Self{ hash: god.hash(), god: true, recruited: false, female }
    }
    pub fn from_unit(unit: engage_il2cpp::app::Unit) -> Self {
        let edit_gender = unit.m_edit().m_gender().value;
        let female = if edit_gender != 0 { edit_gender == 2 } else { unit.get_dress_gender().value == 2 };
        let recruited = if unit.get_force().is_null() { false } else { (1 << unit.get_force_type().value) & 9 != 0 };
        Self{ recruited, female, hash: unit.get_person().hash(), god: false, }
    }
    pub fn try_get_unit(&self) -> Option<engage_il2cpp::app::Unit> {
        self.try_get_person().and_then(|p|{
            let unit = engage_il2cpp::app::UnitPool::get_from_person(p, false);
            if unit.is_null() { None } else { Some(unit) }
        })
    }
    pub fn try_get_god(&self) -> Option<engage_il2cpp::app::GodData>{
        if self.god {
            let g = engage_il2cpp::app::GodData::try_get_from_hash(self.hash);
            if g.is_null() { None } else { Some(g) }
        } else { None }
    }
    pub fn try_get_person(&self) -> Option<engage_il2cpp::app::PersonData> {
        if !self.god {
            let p = engage_il2cpp::app::PersonData::try_get_from_hash(self.hash);
            if p.is_null() { None } else { Some(p) }
        } else { None }
    }
    pub fn get_name(&self) -> Option<unity2::Il2CppString> {
        self.try_get_unit().map(|v| v.get_name() )
            .or_else(|| self.try_get_god().map(|v| engage_il2cpp::app::Mess::get(v.get_mid())))
            .or_else(|| self.try_get_person().map(|v| engage_il2cpp::app::Mess::get(v.get_name() )))
    }
    pub fn get_result(&self, hub: bool) -> AssetTable_Result {
        let default_conditions = engage_il2cpp::combat::CharacterAppearance::get_constions(get_default_asset_conditions());
        if hub {
            self.try_get_unit()
                .map(|u| AssetTable_Result::get_for_accessory(u))
                .or_else(|| self.try_get_person().map(|p| AssetTable_Result::get_for_kizuna(p.get_pid(), default_conditions)))
                .or_else(|| self.try_get_god().map(|g| AssetTable_Result::get_for_hub_2(g)))
        } else {
            self.try_get_unit()
                .map(|u| AssetTable_Result::get_from_unit(AssetTable_Modes::combat(), u, default_conditions))
                .or_else(|| self.try_get_person().map(|p| AssetTable_Result::get_for_kizuna(p.get_pid(), default_conditions)))
                .or_else(|| self.try_get_god().map(|g| AssetTable_Result::get_for_hub_2(g)))
        }.unwrap_or_else(|| AssetTable_Result::get_from_pid(AssetTable_Modes::combat(), "PID_リュール", default_conditions))
    }
}
pub struct ShopUnitSelect;
impl ShopUnitSelect {
    pub fn get_hub_shop_sequence() -> Option<engage_il2cpp::app::HubAccessoryShopSequence> {
        let room = engage_il2cpp::app::HubAccessoryRoom::get_instance();
        if room.is_null() { None } else { room.get_child().try_cast::<engage_il2cpp::app::HubAccessoryShopSequence>() }
    }
    pub fn get_class() -> Class {
        static CLASS: OnceLock<Class> = OnceLock::new();
        *CLASS.get_or_init(|| {
            let klass = Class::try_lookup("App", "ShopUnitSelectMenuItem").unwrap().clone_for_override();
            let klass_raw = klass.raw_mut();
            let vtable = klass_raw.get_vtable_mut();
            vtable[10].method_ptr = Self::on_build_menu_item_content as _;
            vtable[12].method_ptr = Self::on_select as _;
            vtable[18].method_ptr = Self::a_call as _;
            vtable[19].method_ptr = Self::b_call as _;
            klass
        })
    }
    pub fn a_call(this: ShopUnitSelectMenuItem, _: unity2::OptionalMethod) -> BasicMenuResult {
        let hash = unity2::field_get_value_at_offset::<i32>(this, 0x64);
        if UnitAssetMenuData::set_by_hash(hash) {
            if let Some(shop) = Self::get_hub_shop_sequence() { shop.set_m_shop_unit_select_menu_result(BasicMenu_Result{value: 129}); }
            UnitAssetMenuData::get().unit_select_index = this.get_index();
            BasicMenuResult::new().with_se_decide(true).with_close_this(true)
        }
        else { BasicMenuResult::se_miss() }
    }
    pub fn b_call(_: ShopUnitSelectMenuItem, _: unity2::OptionalMethod) -> BasicMenuResult {
        UnitAssetMenuData::get().preview.person = 0;
        if let Some(shop) = Self::get_hub_shop_sequence() {
            shop.set_m_shop_unit_select_menu_result(BasicMenu_Result{value: 513});
            shop.set_m_shop_menu_result(AccessoryShopTopMenu_Result2::end());
            shop.m_accessory_shop_unit_select_root().m_accessory_equipment_info_window().close();
        }
        BasicMenuResult::new().with_se_decide(true).with_close_this(true)
    }
    pub fn on_select(this: ShopUnitSelectMenuItem, _: unity2::OptionalMethod) {
        IBasicMenuItemMethods::on_select(this);
        let select = &mut UnitAssetMenuData::get().unit_select;
        select.selected = Some(this.get_index());
        let default_conditions = engage_il2cpp::combat::CharacterAppearance::get_constions(get_default_asset_conditions());
        if let Some(select) = select.get_selected() {
            if let Some(unit) = select.try_get_unit() {
                UnitAssetMenuData::set_unit(unit);
                // CustomAssetMenu::set_unit_name(unit.get_name());
                let sequence = engage_il2cpp::app::GameUserData::get_instance().get_sequence().value;
                let result =
                    if sequence != 4 { AssetTable_Result::get_for_kizuna(unit.get_pid(), default_conditions) }
                    else { AssetTable_Result::get_for_accessory(unit) };
                result.set_left_hand("null");
                result.set_right_hand("null");
                hub_room_set_by_result(Some(result), ReloadType::All);
            }
            else if let Some(god) = select.try_get_god() {
                UnitAssetMenuData::set_god(god);
                let result = AssetTable_Result::get_for_hub_2(god);
                hub_room_set_by_result(Some(result), ReloadType::All);
            }
            else if let Some(person) = select.try_get_person() {
                UnitAssetMenuData::set_by_hash(person.hash());
                // CustomAssetMenu::set_unit_name(person.get_name());
                let result = AssetTable_Result::get_for_kizuna(person.get_pid(), default_conditions);
                hub_room_set_by_result(Some(result), ReloadType::All);
            }
            if let Some(shop) = Self::get_hub_shop_sequence() {
                EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Assets)
                    .change_equipment_box(shop.m_accessory_shop_unit_select_root().m_accessory_equipment_info_window());
            }
            else { EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Assets).update(); }
        }
    }
    pub fn on_build_menu_item_content(this: ShopUnitSelectMenuItem, _: unity2::OptionalMethod) {
        if let Some(content) = this.get_menu_item_content().try_cast::<ShopUnitSelectMenuItemContent>() {
            set_name_sprite(content, this);
        }
    }
}
pub fn set_name_sprite(content: ShopUnitSelectMenuItemContent, item: ShopUnitSelectMenuItem){
    if content.is_null() || item.is_null() { return; }
    let hash = unity2::field_get_value_at_offset::<i32>(item, 0x64);
    let mut name: Option<unity2::Il2CppString> = None;
    let mut sprite: Option<engage_il2cpp::unity_engine::Sprite> = None;
    let god = engage_il2cpp::app::GodData::try_get_from_hash(hash);
    if !god.is_null() {
        let s = engage_il2cpp::app::FaceThumbnail::get_3(god);
        if !s.is_null() { sprite = Some(s); }
        name = Some(engage_il2cpp::app::Mess::get(god.get_mid()));
    }
    else {
        let person = engage_il2cpp::app::PersonData::try_get_from_hash(hash);
        if !person.is_null() {
            let unit = engage_il2cpp::app::UnitPool::get_hero(false);
            if !unit.is_null() {
                let s = engage_il2cpp::app::FaceThumbnail::get(unit);
                if !s.is_null() { sprite = Some(s); }
                name = Some(engage_il2cpp::app::Mess::get(unit.get_name()));
            }
            else {
                let s = engage_il2cpp::app::FaceThumbnail::get_2(person);
                if !s.is_null() { sprite = Some(s); }
                let m = person.get_name();
                if !m.is_null() { name = Some(engage_il2cpp::app::Mess::get(m)); }
            }
        }
    }
    if let Some(name) = name { content.m_setter().m_unit_name().set_m_text(name); }
    if let Some(sprite) = sprite {
        content.m_setter().m_face().set_sprite(sprite);
        content.m_setter().m_face().get_game_object().set_active(true);
    }
}
pub fn shop_unit_select_menu_item_content_build(this: ShopUnitSelectMenuItemContent, item: ShopUnitSelectMenuItem, _: unity2::OptionalMethod) {
    this.build(item);
    set_name_sprite(this, item);
}

pub extern "C" fn create_accessory_unit_select(this: engage_il2cpp::app::HubAccessoryShopSequence, _: unity2::OptionalMethod) {
    this.create_shop_unit_select_menu();
    if !this.get_child().is_null() {
        if let Some(menu) = this.get_child().try_cast::<ShopUnitSelectMenu>() {
            let menu_list = menu.m_full_menu_item_list();
            menu_list.clear();
            let menu_data = UnitAssetMenuData::get();
            menu_data.is_hub = engage_il2cpp::app::GameUserData::get_instance().get_sequence().value == 4;
            menu_data.unit_select.init();
            menu_data.unit_select.list.iter().for_each(|v|{
                let item = ShopUnitSelectMenuItem::instantiate().unwrap();
                item.rebind_class(ShopUnitSelect::get_class());
                IBasicMenuItemMethods::ctor(item);
                unity2::field_set_value_at_offset(item, 0x64, v.hash);
                UnitAssetMenuData::get_by_person_data(v.hash, true);
                menu_list.add(BasicMenuItem::from(item));
            });
            menu.set_select_index(menu_data.unit_select_index);
        }
        EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Assets)
            .change_equipment_box(this.m_accessory_shop_unit_select_root().m_accessory_equipment_info_window());
    }
}