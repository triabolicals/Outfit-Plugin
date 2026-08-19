use crate::{OUTPUT_ASSET_TABLE_DIR};
use super::*;
use std::io::Write;
use std::path::Path;
use engage::{
	app::{
		assettable::*, jobdata::*,
		IGodDataMethods, IStructData_1Methods, IUnitMethods
	}
};
use crate::room::CustomHubAccessoryRoom;

pub fn get_next_filename(dir: &str, stem: &String, ext: &str) -> String {
	let stem = stem.replace("é", "e");
	let mut file_path = format!("{}{}.{}", dir, stem, ext);
	if Path::new(file_path.as_str()).exists() {
		let mut c = 1;
		loop {
			file_path = format!("{}{}-{}.{}", dir, stem, c, ext);
			if Path::new(file_path.as_str()).exists() { c += 1; }
			else { break; }
		}
	}
	file_path
}
pub fn output_unit_result(preview: bool) -> (String, String, bool) {
	let person_hash = UnitAssetMenuData::get_preview().person;
	let emblem = UnitAssetMenuData::get().god_mode;
	let name =
		if emblem {
			let god = engage::app::GodData::try_get_from_hash(person_hash);
			if !god.is_null() { engage::app::Mess::get(god.get_mid()).to_rust_string() }
			else { format!("Emblem_{}", person_hash) }
	}
	else {
		UnitAssetMenuData::get_shop_unit()
			.map(|v| format!("{}", v.get_name()))
			.unwrap_or(format!("Unit_{}", person_hash))
	};
	let filename1 = get_next_filename(INPUT_DIR, &name, "txt");
	if let Ok(mut file) = std::fs::File::options().create(true).write(true).truncate(true).open(filename1.as_str()){
		writeln!(file, "Outfit Plugin\nVersion: {}", UnitAssetData::version()).unwrap();
		if preview {
			writeln!(file, "Preview").unwrap();
			if let Some(character) = CustomHubAccessoryRoom::get_character().and_then(|c| PlayerOutfitData::from_character(c)){
				writeln!(file, "{}", character.to_string(person_hash)).unwrap();
			}
			else { writeln!(file, "{}", UnitAssetMenuData::get_preview().preview_data.out(person_hash, preview)).unwrap(); }
		}
		else { writeln!(file, "{}", UnitAssetMenuData::get_preview().preview_data.out(person_hash, preview)).unwrap(); }
	}
	let filename = get_next_filename(OUTPUT_ASSET_TABLE_DIR, &name, "txt");
	if let Ok(mut file) = std::fs::File::options().create(true).write(true).truncate(true).open(filename.as_str()){
		let entry = result_to_string(UnitAssetMenuData::get_result(), 2);
		writeln!(&mut file, "{}\n", entry).unwrap();
		return (filename, filename1, true);
	}
	(filename, filename1, false)
}
pub fn output_job_asset_data() {
	let filename = "sd:/Outfits/JobDressData.txt";
	if let Ok(mut file) = std::fs::File::options().create(true).write(true).truncate(true).open(filename) {
		let db = get_outfit_data();
		db.dress.job.iter().for_each(|j| {
			let name = Mess::get(JobData::try_get_from_hash(j.hash).get_name());
			writeln!(file, "Job: {} [{}] {}", name, if j.gender.value == 1 { "Male" } else { "Female" }, j.hash).unwrap();
			writeln!(file, "\tMount: {}", j.mount).unwrap();
			writeln!(file, "\tDress: {}", j.dress_model).unwrap();
			if let Some(ride) = j.ride_dress.as_ref() { writeln!(file, "\tRide Model: {}", ride).unwrap(); }
			if let Some(ride) = j.ride_body.as_ref() { writeln!(file, "\tRide Body Model: {}", ride).unwrap(); }


		});
	}
}
fn il2str_or_blank(str: Il2CppString) -> String {
	if !str.is_null() { str.to_rust_string() } else {"".to_string() }
}
pub fn result_to_string(result: engage::app::AssetTable_Result, mode: i32) -> String {
	let mut out = format!("			<Param Out=\"\" PresetName=\"\" Mode=\"{}\" Conditions=\"\" BodyModel=", mode);
    out.push_str(format!("\"{}\" DressModel=", il2str_or_blank(result.get_body_model())).as_str());
	out.push_str(format!("\"{}\" ", il2str_or_blank(result.get_dress_model())).as_str());
	for x in 0..4 {
		let color = get_result_color(result, 4+x);
		let r = if color.r >= 1.0 { 255 } else { ( color.r * 255.0 ) as u8 };
		let g = if color.g >= 1.0 { 255 } else { ( color.g * 255.0 ) as u8 };
		let b = if color.b >= 1.0 { 255 } else { ( color.b * 255.0 ) as u8 };
		let color_str = format!("{}R=\"{}\" {}G=\"{}\" {}B=\"{}\" ", COLOR_MASK[x], r, COLOR_MASK[x], g, COLOR_MASK[x], b);
		out.push_str(&color_str);
	}
    out.push_str(format!("HeadModel=\"{}\" HairModel=", il2str_or_blank(result.get_head_model())).as_str());
	out.push_str(format!("\"{}\" ", result.get_hair_model()).as_str());
	for x in 0..4 {
		let color = get_result_color(result, x);
		let r = if color.r >= 1.0 { 255 } else { ( color.r * 255.0 ) as u8 };
		let g = if color.g >= 1.0 { 255 } else { ( color.g * 255.0 ) as u8 };
		let b = if color.b >= 1.0 { 255 } else { ( color.b * 255.0 ) as u8 };
		let color_str = format!("{}R=\"{}\" {}G=\"{}\" {}B=\"{}\" ", COLOR_MASK[x], r, COLOR_MASK[x], g, COLOR_MASK[x], b);
		out.push_str(&color_str)
	}
    out.push_str(format!("RideModel=\"{}\" RideDressModel=", il2str_or_blank(result.get_ride_model())).as_str());
	out.push_str(format!("\"{}\" LeftHand=", il2str_or_blank(result.get_ride_dress_model())).as_str());
	out.push_str(format!("\"{}\" RightHand=", il2str_or_blank(result.get_left_hand())).as_str());
	out.push_str(format!("\"{}\" Trail=", il2str_or_blank(result.get_right_hand())).as_str());
	out.push_str(format!("\"{}\" ", il2str_or_blank(result.get_magic())).as_str());
	let mut count = 0;
	result.m_accessory_dictionary().iter()
		.filter(|(k, i)| !k.is_null() && !i.is_null() && !i.get_locator().is_null() && !i.get_model().is_null())
		.for_each(|(_k, i)|{
			if count < 8 {
				count += 1;
				out.push_str(format!("Acc{}.Locator=\"{}\" Acc{}.Model=\"{}\" ", count, i.get_locator(), count, i.get_model()).as_str());
			}
		});
	if count < 8 { for x in count..8 { out.push_str(format!("Acc{}.Locator=\"\" Acc{}.Model=\"\" ", x+1, x+1).as_str()); } }
	out.push_str(format!("BodyAnim=\"{}\" ", il2str_or_blank(result.get_body_anim())).as_str());
	out.push_str(format!("InfoAnim=\"{}\" ", il2str_or_blank(result.m_info_anim())).as_str());
	out.push_str(format!("TalkAnim=\"{}\" ", il2str_or_blank(result.m_talk_anim())).as_str());
	out.push_str(format!("DemoAnim=\"{}\" ", il2str_or_blank(result.m_demo_anim())).as_str());
	out.push_str(format!("HubAnim=\"{}\" ",il2str_or_blank(result.m_hub_anim())).as_str());
	for x in 0..9 { out.push_str(format!("{}=\"{:.2}\" ", SCALE[x], get_result_scale_f32(result, x)).as_str()); }
	out.push_str(format!("{}=\"{:.2}\" ", SCALE[9], get_result_scale_f32(result,  12)).as_str());   // VolumeArm -> VolumeBaseArms
	out.push_str(format!("{}=\"{:.2}\" ", SCALE[10],get_result_scale_f32(result, 13)).as_str());
	out.push_str(format!("{}=\"{:.2}\" ", SCALE[11], get_result_scale_f32(result, 9)).as_str());
	out.push_str(format!("{}=\"{:.2}\" ", SCALE[12], get_result_scale_f32(result, 10)).as_str());
	out.push_str(format!("{}=\"{:.2}\" ", SCALE[13], get_result_scale_f32(result, 11)).as_str());
	for x in 14..19 { out.push_str(format!("{}=\"{:.2}\" ", SCALE[x], get_result_scale_f32(result, x)).as_str()); }
	let voice = result.get_sound();
	out.push_str(format!("Voice=\"{}\" ", il2str_or_blank(voice.voice_id)).as_str());
	out.push_str(format!("FootStep=\"{}\" ", il2str_or_blank(voice.footstep_id )).as_str());
	out.push_str(format!("Material=\"{}\" ", il2str_or_blank(voice.material_id)).as_str());
	out.push_str("Comment=\"Generated by the Unit Asset/Outfit Plugin\" />\n");
	out
}