use engage::app::{ISortieSelectionUnitManagerMethods, hubsequence::*, hublocatorgroup::*, hubplayercontroller::*, ISingletonProcInst_1Methods, AccessoryShopChangeRoot, GameUserData, HubSequence, IGameUserDataMethods, IMapMindMethods, ISingletonClass_1Methods, MapMind, Proc, ProcBoolMethod, ProcDesc, ProcInst, ProcVoidMethod, ResourceManager_2, Force, Force_Type, UnitFor, UnitPool, IUnitMethods, SortieSelectionUnitManager, BasicMenu, UnitInfo, UnitInfo_Side, ISortieSequenceUnitSelect, IBasicMenu, ISortieSequenceUnitSelectMethods, ISortieSelectionUnitManager, UnitSelectMenu, TitleBar, IUnitSelectMenuMethods, IBasicMenuMethods, IBasicMenuItemMethods, ITitleBarMethods};
use engage::{BasicMenuExt, ForceExt, List_1Ext, ProcBoolMethodExt, ProcVoidMethodExt};
use engage::app::procinst::*;
use engage::unity_engine::IComponentMethods;
use unity::{Array, Cast, ClassIdentity, FromIlInstance, IlNull, OptionalMethod};
use engage::unity_engine::IGameObjectMethods;
use crate::{CustomAssetMenu, UnitAssetMenuData};
pub struct OutfitSequence;

impl OutfitSequence {
    pub fn create_bind(proc: impl Into<ProcInst>) -> Option<ProcInst>{
        if GameUserData::get_instance().get_sequence().value == 3 {
            let unit = MapMind::get_instance().get_unit();
            if !unit.is_null() {
                let new_proc = ProcInst::new();
                UnitAssetMenuData::set_unit(unit);
                let descs = OutfitSequence::get_outfit_descs(new_proc);
                new_proc.create_bind(proc, descs, "OutfitSequence");
                return Some(new_proc);
            }
        }
        else {
            let sortie = SortieSelectionUnitManager::get_instance();
            if !sortie.is_null() {
                let unit = sortie.get_unit();
                if !unit.is_null() {
                    let new_proc = ProcInst::new();
                    UnitAssetMenuData::set_unit(unit);
                    let descs = OutfitSequence::get_outfit_descs(new_proc);
                    new_proc.create_bind(proc, descs, "OutfitSequence");
                    return Some(new_proc);
                }
            }
        }
        None
    }
    pub fn get_hub_locator_group() -> Option<HubLocatorGroup> {
        let hub_sequence = HubSequence::get_instance();
        if !hub_sequence.is_null() {
            let locator_group = hub_sequence.get_locator_group();
            if !locator_group.is_null() { Some(locator_group) } else { None }
        } else { None }
    }
    pub fn get_player_controller() -> Option<HubPlayerController> {
        let hub_sequence = HubSequence::get_instance();
        if !hub_sequence.is_null() {
            let locator_group = hub_sequence.get_player();
            if !locator_group.is_null() { Some(locator_group) } else { None }
        } else { None }
    }
    pub fn get_outfit_descs(proc: ProcInst) -> Array<ProcDesc> {
        let descs =
            [
                Proc::label(0),
                Proc::call_2(ProcVoidMethod::from_fn(proc, Self::load_accessory_resources).unwrap()),
                Proc::wait_while_true_2(ProcBoolMethod::from_fn(proc, Self::is_loading).unwrap()),
                Proc::call_2(ProcVoidMethod::from_fn(proc, Self::save_accessories).unwrap()),
                Proc::label(1),
                Proc::call_2(ProcVoidMethod::from_fn(proc, Self::create_unit_info_bind).unwrap()),
                Proc::label(2),
                Proc::call_2(ProcVoidMethod::from_fn(proc, Self::restore_player_controller).unwrap()),
                Proc::wait_while_true_2(ProcBoolMethod::from_fn(proc, Self::character_loading).unwrap()),
                Proc::call_2(ProcVoidMethod::from_fn(proc, Self::restore_others).unwrap()),
                Proc::wait_while_true_2(ProcBoolMethod::from_fn(proc, Self::character_loading).unwrap()),
                Proc::call_2(ProcVoidMethod::from_fn(proc, Self::reset_look_at).unwrap()),
                Proc::label(3),
                Proc::call_2(ProcVoidMethod::from_fn(proc, Self::unload_accessory_resources).unwrap()),
                Proc::call_2(ProcVoidMethod::from_fn(proc, Self::restore_unit_info).unwrap()),
                Proc::label(4),
                Proc::end(),
            ];
        Array::from_slice(&descs).unwrap()
    }
    extern "C" fn restore_unit_info(this: ProcInst, _: OptionalMethod) {
        let sequence = GameUserData::get_instance().get_sequence().value;
        if sequence == 2 || sequence == 3 {
            UnitPool::get_force(0).iter().for_each(|unit| unit.reload_actor());
        }
        if let Some(menu) = this.get_super().try_cast::<BasicMenu>() {
            IBasicMenuMethods::open_anime_all(menu);
        }

    }
    extern "C" fn reset_look_at(_this: ProcInst, _: OptionalMethod) {
        if let Some(group) = Self::get_hub_locator_group() { group.reset_look_at(); }
        if let Some(player) = Self::get_player_controller() { player.init_look_at_target(); }
    }
    extern "C" fn save_accessories(_this: ProcInst, _: OptionalMethod) {
        if let Some(group) = Self::get_hub_locator_group() { group.save_accessory(); }
        if let Some(player) = Self::get_player_controller() { player.save_accessory(); }
    }
    extern "C" fn restore_others(_this: ProcInst, _: OptionalMethod) {
        if let Some(group) = Self::get_hub_locator_group() {
            group.set_active(true);
            group.restore_accessory();
        }
    }
    extern "C" fn character_loading(_this: ProcInst, _: OptionalMethod) -> bool {
        ResourceManager_2::is_loading() ||
            Self::get_hub_locator_group().is_some_and(|v| v.is_character_loading()) ||
            Self::get_player_controller().is_some_and(|v|v.get_is_character_loading())
    }
    extern "C" fn restore_player_controller(_this: ProcInst, _: OptionalMethod) {
        if let Some(player) = Self::get_player_controller() { player.restore_accessory(); }
    }
    extern "C" fn load_accessory_resources(_this: ProcInst, _: OptionalMethod) { AccessoryShopChangeRoot::load_prefab_async(); }
    extern "C" fn unload_accessory_resources(_this: ProcInst, _: OptionalMethod) { AccessoryShopChangeRoot::unload_prefab(); }
    extern "C" fn is_loading(_this: ProcInst, _: OptionalMethod) -> bool { AccessoryShopChangeRoot::is_loading_prefab() }
    extern "C" fn create_unit_info_bind(proc: ProcInst, _: OptionalMethod) {
        if let Some(unit) = UnitAssetMenuData::get_unit(){ CustomAssetMenu::create_bind_unit_info(proc, unit); }
        else { proc.jump(4); }
    }
}

