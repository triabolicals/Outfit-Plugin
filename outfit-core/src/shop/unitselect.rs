use std::sync::OnceLock;
use engage::{
    combat::CharacterAppearance, gameuserdata::GameUserData,
    god::GodPool, unityengine::Component,
    menu::{
        menus::shop::shopunitselect::*,
        menu_item::{MenuItem, MenuItemContent}, BasicMenu, BasicMenuResult
    },
    util::get_singleton_proc_instance,
    mess::Mess, unit::{Unit, UnitFor, UnitPool},
    proc::Bindable, spriteatlasmanager::FaceThumbnail, 
    gamedata::{PersonData, Gamedata, GodData, assettable::AssetTableResult},
    sequence::hubaccessory::{room::HubAccessoryRoom, HubAccessoryShopSequence},
};
use unity::{prelude::*, engine::{ui::IsImage, Color}};
use crate::{EquipmentBoxMode, EquipmentBoxPage, UnitAssetMenuData};
use crate::room::ReloadType;
use crate::shop::room::hub_room_set_by_result;
static SHOP_UNIT_SELECT_CLASS: OnceLock<&'static Il2CppClass> = OnceLock::new();

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
        if let Some(unit) = UnitPool::get_hero(false) {
            self.list.push(UnitSelect::from_unit(unit));
            let mut start = unit;
            while let Some(u) = UnitFor::get_next_by_force(start, 9){
                if !self.list.iter().any(|v| v.hash == u.person.parent.hash) { self.list.push(UnitSelect::from_unit(u)); }
                start = u;
            }
        }
        data.iter().for_each(|data| {
            if !self.list.iter().any(|v| v.hash == data.person) {
                if let Some(person) = PersonData::try_get_hash(data.person){ self.list.push(UnitSelect::from_person(person)); }
            }
        });
        GodPool::get_instance().sort
            .iter().filter(|x| x.get_escape() == false && x.data.force_type == 0)
            .for_each(|god_unit| {
                god_unit.data.change_data.iter().for_each(|g| {
                    if !self.list.iter().any(|v| v.hash == g.parent.hash) {
                        self.list.push(UnitSelect::from_god(g));
                    }
                });
            });
    }
    pub fn change(&mut self, next: bool) {
        if let Some(current) = self.selected.as_ref() {
            let size = self.list.len() as i32;
            let new = if next { (*current + 1) % size } else { (*current + size - 1) % size };
            self.selected = Some(new);
        }
        else { self.selected = Some(0); }
    }
    pub fn get_selected(&self) -> Option<UnitSelect> { self.selected.and_then(|v| self.list.get(v as usize).cloned()) }
    pub fn get_result(&self, hub: bool) -> &'static mut AssetTableResult {
        if let Some(result) = self.get_selected().map(|v| v.get_result(hub)) { result }
        else { AssetTableResult::get_from_pid(2, "PID_リュール", CharacterAppearance::get_constions(None)) }
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
    pub fn from_person(person: &PersonData) -> Self {
        let female = if person.flag.value & 32 != 0 { person.gender != 2 }
        else { person.gender == 2 };
        Self{
            hash: person.parent.hash,
            god: false,
            recruited: false,
            female,
        }
    }
    pub fn from_god(god: &GodData) -> Self {
        let female = if god.is_hero() { UnitPool::get_hero(false).map(|u| u.edit.gender == 2).unwrap_or(false) }
        else { god.female != 0 };
        Self{
            hash: god.parent.hash,
            god: true,
            recruited: false,
            female,
        }
    }
    pub fn from_unit(unit: &Unit) -> Self {
        let female = if unit.edit.gender != 0 { unit.edit.gender == 2 }
        else if unit.person.flag.value & 32 != 0 { unit.person.gender != 2 }
        else { unit.person.gender == 2 };

        Self{
            hash: unit.person.parent.hash,
            god: false,
            recruited: unit.force.is_some_and(|v| (1 << v.force_type) & 9 != 0),
            female,
        }
    }
    pub fn try_get_unit(&self) -> Option<&'static mut Unit> { self.try_get_person().and_then(|p| UnitPool::get_from_person(p, false)) }
    pub fn try_get_god(&self) -> Option<&'static GodData> { if self.god { GodData::try_get_hash(self.hash) } else { None } }
    pub fn try_get_person(&self) -> Option<&'static PersonData> { if !self.god { PersonData::try_get_hash(self.hash) } else { None } }
    pub fn get_name(&self) -> Option<&'static mut Il2CppString> {
        self.try_get_unit().map(|v| v.get_name() )
            .or_else(|| self.try_get_god().map(|v| Mess::get(v.mid)))
            .or_else(|| self.try_get_person().map(|v| v.get_name() ))
    }
    pub fn get_result(&self, hub: bool) -> &'static mut AssetTableResult {
        if let Some(result) =  self.try_get_unit()
            .map(|unit|{
                if hub { AssetTableResult::get_for_accessory(unit) }
                else { AssetTableResult::get_from_unit(2, unit, CharacterAppearance::get_constions(None)) }
            }) {
            result
        }
        else if let Some(result) = self.try_get_person().map(|person| AssetTableResult::get_for_kizuna(person.pid, CharacterAppearance::get_constions(None)) ){
            result
        }
        else if let Some(result) =
            self.try_get_god().and_then(|god|
                if hub { Some(AssetTableResult::get_for_hub_god(god)) }
                else {
                    GodPool::try_get(god, false)
                        .map(|g_unit| AssetTableResult::get_from_god_unit(2, g_unit, CharacterAppearance::get_constions(None)))
                }
            )
        {
            result
        }
        else { AssetTableResult::get_from_pid(2, "PID_リュール", CharacterAppearance::get_constions(None)) }
    }
}
#[unity::class("App", "ShopUnitSelectMenuItem")]
pub struct ShopUnitSelectMenuItem2 {
    pub menu: &'static mut BasicMenu<ShopUnitSelectMenuItem2>,
    pub menu_item_content: Option<&'static mut ShopUnitSelectMenuItemContent>,
    pub name: &'static Il2CppString,
    pub index: i32,
    pub full_index: i32,
    pub attribute: i32,
    pub cursor_color: Color,
    pub active_active: Color,
    pub inactive_active: Color,
    pub god_hash: i32,
    pub unit: Option<&'static mut Unit>,
    pub decided_handler: Option<&'static mut ShopUnitSelectMenuDecideHandler>,
    pub select_handler: Option<&'static mut ShopUnitSelectMenuSelectHandler>,
}
impl MenuItem for ShopUnitSelectMenuItem2 {}
impl ShopUnitSelectMenuItem2 {
    pub fn create_class() -> &'static Il2CppClass {
        let klass = Il2CppClass::from_name("App", "ShopUnitSelectMenuItem").unwrap().clone();
        let vtable = klass.get_vtable_mut();
        vtable[10].method_ptr = Self::on_build_menu_item_content as _;
        vtable[12].method_ptr = Self::on_select as _;
        vtable[18].method_ptr = Self::a_call as _;
        vtable[19].method_ptr = Self::b_call as _;
        klass
    }
    pub fn b_call(this: &mut ShopUnitSelectMenuItem2, _optional_method: OptionalMethod) -> BasicMenuResult {
        if let Some(shop) = HubAccessoryRoom::get_instance().and_then(|v| v.get_child().map(|v| v.cast_mut::<HubAccessoryShopSequence>())){
            shop.shop_menu_result = 1;
            shop.shop_unit_select_menu_result = 513;
            shop.select_menu_scroll_index = this.menu.scroll_index;
            shop.unit_select_root.equipment.close();
            UnitAssetMenuData::get().preview.person = 0;
        }
        BasicMenuResult::new().with_se_decide(true).with_close_this(true)
    }
    pub fn a_call(this: &mut ShopUnitSelectMenuItem2, _optional_method: OptionalMethod) -> BasicMenuResult {
        if UnitAssetMenuData::set_by_hash(this.god_hash) {
            if let Some(shop) = HubAccessoryRoom::get_instance().and_then(|v| v.get_child().map(|v| v.cast_mut::<HubAccessoryShopSequence>())){
                shop.shop_menu_result = 1;
                shop.shop_unit_select_menu_result = 129;
                shop.select_menu_scroll_index = this.menu.scroll_index;
            }
        }
        else if let Some((select_handler, unit)) = this.decided_handler.as_ref().zip(this.unit.as_ref()){
            select_handler.invoke(0x81, unit, this.menu.scroll_index);
        }
        BasicMenuResult::new().with_se_decide(true).with_close_this(true)
    }
    pub fn on_select(this: &ShopUnitSelectMenuItem2, _optional_method: OptionalMethod) {
        this.on_select_base();
        if let Some(room) = HubAccessoryRoom::get_instance() {
            let select = &mut UnitAssetMenuData::get().unit_select;
            select.selected = Some(this.index);
            if let Some(select) = select.get_selected() {
                if let Some(unit) = select.try_get_unit() {
                    UnitAssetMenuData::set_unit(unit);
                    let sequence = GameUserData::get_sequence();
                    let result = if sequence == 2 || sequence == 3 || sequence == 6 { AssetTableResult::get_from_unit(2, unit, CharacterAppearance::get_constions(None)) }
                    else { AssetTableResult::get_for_accessory(unit) };
                    room.set_unit_core(unit, 0, true);
                    result.left_hand = "null".into();
                    result.right_hand = "null".into();
                    hub_room_set_by_result(Some(result), ReloadType::All);
                    EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Assets).update();
                }
                else if let Some(god) = select.try_get_god() {
                    UnitAssetMenuData::set_god(god);
                    let result = AssetTableResult::get_for_hub_god(god);
                    room.last_pid = god.gid.to_string().into();
                    let appearance = CharacterAppearance::create_from_result(result, 1);
                    room.loading_appearance = Some(appearance);
                    room.load_character(appearance, god.gid);
                    if let Some(shop) = get_singleton_proc_instance::<HubAccessoryShopSequence>() {
                        EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Assets)
                            .change_equipment_box(shop.unit_select_root.equipment);
                    }
                    else { EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Assets).update(); }
                }
                else if let Some(person) = select.try_get_person() {
                    UnitAssetMenuData::set_by_hash(person.parent.hash);
                    let result = AssetTableResult::get_for_kizuna(person.pid, CharacterAppearance::get_constions(None));
                    room.last_pid = person.pid.to_string().into();
                    let appearance = CharacterAppearance::create_from_result(result, 1);
                    room.loading_appearance = Some(appearance);
                    room.load_character(appearance, person.pid);
                }
                if let Some(shop) = get_singleton_proc_instance::<HubAccessoryShopSequence>() {
                    EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Assets)
                        .change_equipment_box(shop.unit_select_root.equipment);
                }
                else { EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Assets).update(); }
            }
            else {
                if let Some((select_handler, unit)) = this.select_handler.as_ref().zip(this.unit.as_ref()){ select_handler.invoke(unit); }
            }
        }
    }
    pub fn on_build_menu_item_content(this: &mut ShopUnitSelectMenuItem2, _optional_method: OptionalMethod) {
        let hash = this.god_hash;
        if let Some(content) = this.menu_item_content.as_mut(){
            if let Some(god) = GodData::try_get_hash(hash){

                content.setter.unit_name.set_text(Mess::get(god.mid), true);
                if let Some(face) = FaceThumbnail::get_god(god) { content.setter.face.set_sprite(face); }
            }
            else if let Some(person) = PersonData::try_get_hash(hash) {
                if let Some(unit) = UnitPool::get_from_person(person, false) {
                    content.setter.unit_name.set_text(unit.get_name(),true);
                }
                else { content.setter.unit_name.set_text(person.get_name(), false); }
                if let Some(face) = FaceThumbnail::get_from_person(person) {
                    if let Some(go) = Component::get_go(content.setter.face){ go.set_active2(true); }
                    content.setter.face.set_sprite(face);
                }
            }
        }
    }
}
pub fn shop_unit_select_menu_item_content_build(this: &mut ShopUnitSelectMenuItemContent, item: &ShopUnitSelectMenuItem2, _method_info: OptionalMethod) {
    let item2 = unsafe { std::mem::transmute::<_, &ShopUnitSelectMenuItem>(item) };
    this.build(item2);
    if let Some(god) = GodData::try_get_hash(item.god_hash) {
        this.setter.unit_name.set_text(Mess::get(god.mid), true);
        if let Some(face) = FaceThumbnail::get_god(god) {
            if let Some(go) = Component::get_go(this.setter.face){ go.set_active2(true); }
            this.setter.face.set_sprite(face);
        }
    }
    else if let Some(person) = PersonData::try_get_hash(item.god_hash) {
        if let Some(unit) = UnitPool::get_from_person(person, false) {
            this.setter.unit_name.set_text(unit.get_name(),true);
        }
        else { this.setter.unit_name.set_text(person.get_name(), false); }
        if let Some(face) = FaceThumbnail::get_from_person(person) {
            if let Some(go) = Component::get_go(this.setter.face){ go.set_active2(true); }
            this.setter.face.set_sprite(face);
        }
    }
}

pub extern "C" fn create_accessory_unit_select(this: &mut HubAccessoryShopSequence, _optional_method: OptionalMethod) {
    this.shop_menu_result = 1;
    this.create_shop_unit_select_menu();
    if let Some(menu) = this.proc.child.as_mut().map(|v| v.cast_mut::<BasicMenu<ShopUnitSelectMenuItem2>>()) {
        menu.full_menu_item_list.clear();
        UnitAssetMenuData::get().is_hub = GameUserData::get_sequence() == 4;
        UnitAssetMenuData::get().unit_select.init();
        UnitAssetMenuData::get().unit_select.list.iter().for_each(|v|{
            let item = SHOP_UNIT_SELECT_CLASS
                .get_or_init(|| ShopUnitSelectMenuItem2::create_class())
                .instantiate_as::<ShopUnitSelectMenuItem2>().unwrap();
            item.ctor_base();
            item.god_hash = v.hash;
            UnitAssetMenuData::get_by_person_data(v.hash, true);
            menu.add_item(item);
        });
        menu.proc.desc_index = 0;
        EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Assets)
            .change_equipment_box(this.unit_select_root.equipment);
    }
}