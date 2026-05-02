use engage::{
    gamedata::accessory::AccessoryData,
    gamedata::assettable::{AssetTable, AssetTableResult, AssetTableStaticFields},
    gamedata::{Gamedata, PersonData},
    unit::{Unit, UnitAccessoryList}
};

pub struct AccessoryConditions {
    pub mode_1: Vec<Vec<i32>>,
    pub mode_2: Vec<Vec<i32>>,
}

impl AccessoryConditions {
    pub fn new() -> Self {
        let acc_data = AccessoryData::get_list_mut().unwrap();
        let count = acc_data.len();
        let sf = AssetTableStaticFields::get();
        let list = AssetTable::get_list().unwrap();
        let mut mode_1 = vec![vec![]; count];
        let mut mode_2 = vec![vec![]; count];
        if let Some(causal_lueur) = list.iter().find(|x| x.mode == 2 && x.dress_model.is_some_and(|x| x.contains("WearM_c001"))) {
            mode_2[1].push(causal_lueur.parent.index);
            acc_data[1].condition_gender = 1;
        }
        if let Some(causal_lueur) = list.iter().find(|x| x.mode == 2 && x.dress_model.is_some_and(|x| x.contains("WearF_c051"))) {
            mode_2[2].push(causal_lueur.parent.index);
            acc_data[2].condition_gender = 2;
        }
        let mut conds: [i32; 3] = [AssetTableStaticFields::get_condition_index("体アクセサリ無し"), AssetTableStaticFields::get_condition_index("私服"), 0];
        for i in 1..41 {
            let aid = acc_data[i+2].aid.to_string();
            let pid = aid.replace("AID_", "PID_");
            let pid = pid.trim_end_matches("私服");
            if let Some(person) = PersonData::get(pid) {
                if person.flag.value & 32 != 0 {
                    if person.gender == 1 { acc_data[i+2].condition_gender = 2; }
                    else { acc_data[i+2].condition_gender = 1; }
                }
                else { acc_data[i+2].condition_gender = person.gender; }
            }
            conds[2] = AssetTableStaticFields::get_condition_index(pid);
            if let Some(entry) = sf.search_lists[2].iter().find(|x| conds.iter().any(|c| x.condition_indexes.has_condition_index(*c))){
                mode_2[i+2].push(entry.parent.index);
            }
        }
        for x in 43..count {
            let index = AssetTableStaticFields::get_condition_index(acc_data[x].aid);
            mode_1[x] = sf.search_lists[1].iter()
                .filter(|y| y.condition_indexes.list.iter()
                    .any(|i| i.iter().any(|i2| *i2 == index))).map(|entries| entries.parent.index)
                .collect();
            mode_2[x] = sf.search_lists[2].iter()
                .filter(|y| y.condition_indexes.list.iter()
                    .any(|i| i.iter().any(|i2| *i2 == index))).map(|entries| entries.parent.index)
                .collect();
        }
        Self { mode_1, mode_2 }
    }
    pub fn commit_accessories(&self, result: &mut AssetTableResult, unit: &Unit, mode: i32) {
        let count = UnitAccessoryList::get_count() as usize;
        let hub = AssetTableStaticFields::get_condition_index("私服");
        let sf = AssetTableStaticFields::get();
        let bits = &sf.condition_flags;
        let set = if mode == 2 { &self.mode_2 } else { &self.mode_1 };
        for x in 0..count {
            if x == 4 { continue; }
            let index = unit.accessory_list.unit_accessory_array[x].index;
            if index > 0 {
                if let Some(aid) = AccessoryData::try_index_get(index).map(|a| a.aid) {
                    let aid_con = AssetTableStaticFields::get_condition_index(aid);
                    set[index as usize].iter()
                        .flat_map(|x| AssetTable::try_index_get(*x))
                        .filter(|x| x.condition_indexes.list.iter().all(|x| x.iter().any(|s| bits.bits.get(*s) || *s == hub || *s == aid_con)) || index < 43)
                        .for_each(|x| { Self::apply_accessory(result, x, index < 43); });
                }
            }
        }
    }
    pub fn apply_accessory(result: &mut AssetTableResult, x: &AssetTable, ignore_head: bool) {
        if let Some(body) = x.body_model { result.body_model = body; }
        if let Some(dress) = x.dress_model { result.dress_model = dress; }
        if let Some(hair) = x.hair_model { result.hair_model = hair; }
        if ignore_head { if let Some(head) = x.head_model { result.head_model = head; } }
        if let Some(aoc) = x.info_anim { result.info_anims = Some(aoc); }
        x.accessories.iter().filter(|x| x.locator.is_some()).for_each(|x| { result.commit_accessory(x); });
    }
}