use std::borrow::BorrowMut;
use std::sync::atomic::{AtomicPtr, Ordering};
use smash::{
    *,
    hash40,
    phx::{Hash40, Vector3f},
    lib::{lua_const::*, L2CValue},
    app::{lua_bind::{*, StatusModule::*}, sv_animcmd::{frame, wait}, BattleObjectModuleAccessor},
    lua2cpp::{L2CFighterCommon, L2CAgentBase, L2CFighterBase}
};
use smash_utils::bomaext::BomaExt;
use smashline::*;
use smash_script::{macros::*, notify_event_msc_cmd, lua_args};
use std::vec::Vec;
use smash::lib::lua_const::*;
use crate::utils::*;
use skyline::hooks::{getRegionAddress, Region};
use skyline::libc::*;
use smash::app::{ArticleOperationTarget, enSEType};
use smash_utils::cmdflag::Buttons;
use crate::vegeta_status::*;
use smash_utils::utils::{CancelKind, FIGHTER_MANAGER};
pub static mut CHARGE_TIME:[f32;8] = [0.0;8];
pub static mut TEST: i32 = 0;
pub static mut TEST2: i32 = 0;
pub static mut TEST3: i32 = 0;


pub const FIGHTER_VEGETA_GENERATE_ARTICLE_BBATK: i32 = 0x4;

enum MeshType{
    SS,
    SSB,
    SSBE,
    Base,
    All
}
const ALL_VEGETA_MESHES:[&str;39] = [
    "ken_bingo_mouth", "ken_bingo_mouth2", "ken_bingo_y", "ken_bingo", "ken_bingo_b", "ken_bingo_p", "ken_facen_mouth", "ken_final_mouth", "ken_finalblink",
    "ken_finalblink_b", "ken_finalblink_y", "ken_finalblink_p", "ken_hurt_mouth", "ken_hurtblink", "ken_hurtblink_y", "ken_hurtblink_b", "ken_hurtblink_p", "ken_laugh_mouth",
    "ken_laughblink", "ken_laughblink_y", "ken_laughblink_b","ken_laughblink_p", "ken_openblink", "ken_openblink_y", "ken_openblink_b", "ken_openblink_p", "ken_saiyan_mouth",
    "ken_smile2_mouth", "ken_smile2blink", "ken_smile2blink_y", "ken_smile2blink_b", "ken_smile2blink_p", "ken_smile_mouth", "ken_hair1", "ken_hair2",
    "ken_hair3", "ken_hair4", "ken_hair5", "ken_earring_p",
];


pub unsafe fn disable_all_face_meshes(module_accessor: *mut BattleObjectModuleAccessor){
    for mesh in ALL_VEGETA_MESHES {
        ModelModule::set_mesh_visibility(module_accessor, Hash40::new(mesh), false);
    }
}

pub unsafe fn ue_neutral_face(module_accessor: *mut app::BattleObjectModuleAccessor){
    disable_all_face_meshes(module_accessor);
    ModelModule::set_mesh_visibility(module_accessor, Hash40::new("ken_smile_mouth"), true);
    ModelModule::set_mesh_visibility(module_accessor, Hash40::new("ken_openblink_p"), true);
    ModelModule::set_mesh_visibility(module_accessor, Hash40::new("ken_hair5"), true);
    ModelModule::set_mesh_visibility(module_accessor, Hash40::new("ken_earring_p"), true);
}

pub unsafe fn base_neutral_face(module_accessor: *mut app::BattleObjectModuleAccessor){
    disable_all_face_meshes(module_accessor);
    ModelModule::set_mesh_visibility(module_accessor, Hash40::new("ken_facen_mouth"), true);
    ModelModule::set_mesh_visibility(module_accessor, Hash40::new("ken_openblink"), true);
    ModelModule::set_mesh_visibility(module_accessor, Hash40::new("ken_hair1"), true);
}

pub unsafe fn base_power_attack_face_n(module_accessor: *mut app::BattleObjectModuleAccessor){
    disable_all_face_meshes(module_accessor);
    ModelModule::set_mesh_visibility(module_accessor, Hash40::new("ken_final_mouth"), true);
    ModelModule::set_mesh_visibility(module_accessor, Hash40::new("ken_finalblink"), true);
    ModelModule::set_mesh_visibility(module_accessor, Hash40::new("ken_hair1"), true);
}

pub unsafe fn base_power_charge_face_n(module_accessor: *mut app::BattleObjectModuleAccessor){
    disable_all_face_meshes(module_accessor);
    ModelModule::set_mesh_visibility(module_accessor, Hash40::new("ken_hurt_mouth"), true);
    ModelModule::set_mesh_visibility(module_accessor, Hash40::new("ken_finalblink"), true);
    ModelModule::set_mesh_visibility(module_accessor, Hash40::new("ken_hair1"), true);
}


const ZERO_VECTOR:Vector3f = Vector3f{x: 0.0, y: 0.0, z: 0.0};



pub unsafe fn ki_charge(module_accessor: &mut BattleObjectModuleAccessor){
    if [hash40("appeal_s_l"), hash40("appeal_s_r")].contains(&MotionModule::motion_kind(module_accessor)){
        change_motion(module_accessor, "ki_charge")
    }
    if MotionModule::motion_kind(module_accessor) == hash40("ki_charge") && MotionModule::frame(module_accessor) == 30.0{
        MotionModule::set_rate(module_accessor, 0.0);
        let eff: u32 = EffectModule::req_follow(module_accessor, Hash40::new("sys_magicball_aura"), smash::phx::Hash40::new("rot"), &Vector3f {x: -2.0*PostureModule::lr(module_accessor), y: -3.0, z: 1.0*PostureModule::lr(module_accessor)}, &ZERO_VECTOR, 2.5, true, 0, 0, 0, 0, 0, true, true) as u32;
        EffectModule::set_scale(module_accessor, eff, &Vector3f{
            x: 3.0,
            y: 3.0,
            z: 5.0
        });
        EffectModule::set_add_vel(module_accessor, eff, &mut Vector3f{
            x: 0.0,
            y: 1.0,
            z: 0.0
        });
        EffectModule::set_rate(module_accessor, eff, 0.7);
    }
}

static mut BUTTON_QCF_TIMER:[f32;8] = [0.0;8];
static mut BUTTON_QCB_TIMER:[f32;8] = [0.0;8];
static mut START_TIMER_QCF:[bool;8] = [false;8];
static mut START_TIMER_QCB:[bool;8] = [false;8];
pub unsafe fn qcf_handle(module_accessor: &mut BattleObjectModuleAccessor){
    let entry_id = get_entry_id(module_accessor);
    let status_kind = StatusModule::status_kind(module_accessor);
        
    if is_qcf(module_accessor) {
        START_TIMER_QCF[entry_id] = true;
    }
    if START_TIMER_QCF[entry_id]{
        BUTTON_QCF_TIMER[entry_id] +=1.0
    }
    if START_TIMER_QCF[entry_id] && BUTTON_QCF_TIMER[entry_id] < 20.0{
        if ControlModule::check_button_on(module_accessor, *CONTROL_PAD_BUTTON_ATTACK) &&
                ![FIGHTER_VEGETA_STATUS_KIND_SUPERDASHKICK].contains(&status_kind){
            StatusModule::change_status_request_from_script(module_accessor, FIGHTER_VEGETA_STATUS_KIND_SUPERDASHKICK, true);
        }
    }
    else{
        BUTTON_QCF_TIMER[entry_id] = 0.0;
        START_TIMER_QCF[entry_id] = false;
    }
    if [hash40("superdashkick")].contains(&MotionModule::motion_kind(module_accessor)){
        if MotionModule::is_end(module_accessor){
            enable_gravity(module_accessor)
        }
        else{
            disable_gravity(module_accessor);
        }
    }
}

pub unsafe fn qcb_handle(module_accessor: &mut BattleObjectModuleAccessor){
    let entry_id = get_entry_id(module_accessor);
    let status_kind = StatusModule::status_kind(module_accessor);

    if is_qcb(module_accessor) {
        START_TIMER_QCB[entry_id] = true;
    }
    if START_TIMER_QCB[entry_id]{
        BUTTON_QCB_TIMER[entry_id] +=1.0
    }
    if START_TIMER_QCB[entry_id] && BUTTON_QCB_TIMER[entry_id] < 20.0{
        if ControlModule::check_button_on(module_accessor, *CONTROL_PAD_BUTTON_SPECIAL)
            && ![FIGHTER_VEGETA_STATUS_KIND_GALICK_GUN_HOLD, FIGHTER_VEGETA_STATUS_KIND_GALICK_GUN_START, FIGHTER_VEGETA_STATUS_KIND_GALICK_GUN_FIRE].contains(&status_kind){
            StatusModule::change_status_request_from_script(module_accessor, FIGHTER_VEGETA_STATUS_KIND_GALICK_GUN_START, true);
        }
    }
    else{
        BUTTON_QCB_TIMER[entry_id] = 0.0;
        START_TIMER_QCB[entry_id] = false;
    }
}

    //yamlist.exe asm C:\Users\m\Desktop\ALLSSBUMODS2\VegetaMoveset\Code\motion_list.yml -o C:\Users\m\Desktop\ALLSSBUMODS2\VegetaMoveset\VegetaMovesetAnims\fighter\ken\motion\body\c00\motion_list.bin

pub static mut EFFPOS: smash::phx::Vector3f = smash::phx::Vector3f {x:0.0,y:0.0,z:0.0};
pub static mut EFFROT: smash::phx::Vector3f = smash::phx::Vector3f {x:0.0,y:0.0,z:0.0};


#[weapon_frame(agent = WEAPON_KIND_LUCARIO_AURABALL)]
pub fn hadoken(weapon: &mut L2CFighterBase) {
    unsafe{
        let lua_state = weapon.lua_state_agent;
        let weapon_module_accessor = smash::app::sv_system::battle_object_module_accessor(lua_state);
        let owner_module_accessor = &mut *weapon_module_accessor.get_owner_boma();
        let owner_motion_kind = MotionModule::motion_kind(owner_module_accessor);

        EffectModule::kill_kind(weapon_module_accessor, Hash40::new("lucario_hadoudan_tail"), true, true);
        EffectModule::kill_kind(weapon_module_accessor, Hash40::new("lucario_hadoudan_hold"), true, true);
        EffectModule::kill_kind(weapon_module_accessor, Hash40::new("lucario_hadoudan_max_sign"), true, true);
        EffectModule::kill_kind(weapon_module_accessor, Hash40::new("lucario_hadoudan_max_hold"), true, true);
        EffectModule::kill_kind(weapon_module_accessor, Hash40::new("lucario_hadoudan_bomb"), true, true);
        EffectModule::kill_kind(weapon_module_accessor, Hash40::new("lucario_hadoudan_max_l"), true, true);
        EffectModule::kill_kind(weapon_module_accessor, Hash40::new("lucario_hadoudan_max_r"), true, true);
        EffectModule::kill_kind(weapon_module_accessor, Hash40::new("lucario_hadoudan"), true, true);

        if owner_module_accessor.status_kind() == *FIGHTER_STATUS_KIND_SPECIAL_S{
            if weapon_module_accessor.motion_frame() == 1.0{
                let bbatk = EffectModule::req_follow(weapon_module_accessor, Hash40::new("sys_sscope_bullet_max"), smash::phx::Hash40::new("top"), &ZERO_VECTOR, &ZERO_VECTOR, 3.0, true, 0, 0, 0, 0, 0, true, true) as u32;
                EffectModule::set_rgb(weapon_module_accessor, bbatk, 0.0, 0.8, 13.0);
            }
            acmd!(lua_state, {
                ATTACK(ID=0, Part=0, Bone=hash40("top"), Damage=10.0, Angle=40, KBG=65, FKB=0, BKB=55, Size=8.0, X=0.0, Y=0.0, Z=0.0, X2=LUA_VOID, Y2=LUA_VOID, Z2=LUA_VOID, Hitlag=1.0, SDI=1.0, Clang_Rebound=ATTACK_SETOFF_KIND_ON, FacingRestrict=ATTACK_LR_CHECK_SPEED, SetWeight=false, ShieldDamage=-7, Trip=0.0, Rehit=0, Reflectable=true, Absorbable=true, Flinchless=false, DisableHitlag=false, Direct_Hitbox=false, Ground_or_Air=COLLISION_SITUATION_MASK_GA, Hitbits=COLLISION_CATEGORY_MASK_ALL, CollisionPart=COLLISION_PART_MASK_ALL, FriendlyFire=false, Effect=hash40("collision_attr_aura"), SFXLevel=ATTACK_SOUND_LEVEL_L, SFXType=COLLISION_SOUND_ATTR_KICK, Type=ATTACK_REGION_ENERGY)
            });
        }
        if owner_module_accessor.status_kind() == *FIGHTER_STATUS_KIND_SPECIAL_N{
            if weapon_module_accessor.motion_frame() == 1.0{
                let kiblast = EffectModule::req_follow(weapon_module_accessor, Hash40::new("sys_sscope_bullet"), smash::phx::Hash40::new("top"), &ZERO_VECTOR, &ZERO_VECTOR, 1.9, true, 0, 0, 0, 0, 0, true, true) as u32;
            }
            acmd!(lua_state, {
                ATTACK(ID=0, Part=0, Bone=hash40("top"), Damage=3.0, Angle=40, KBG=65, FKB=0, BKB=55, Size=4.24, X=0.0, Y=0.0, Z=0.0, X2=LUA_VOID, Y2=LUA_VOID, Z2=LUA_VOID, Hitlag=1.0, SDI=1.0, Clang_Rebound=ATTACK_SETOFF_KIND_ON, FacingRestrict=ATTACK_LR_CHECK_SPEED, SetWeight=false, ShieldDamage=-7, Trip=0.0, Rehit=0, Reflectable=true, Absorbable=true, Flinchless=false, DisableHitlag=false, Direct_Hitbox=false, Ground_or_Air=COLLISION_SITUATION_MASK_GA, Hitbits=COLLISION_CATEGORY_MASK_ALL, CollisionPart=COLLISION_PART_MASK_ALL, FriendlyFire=false, Effect=hash40("collision_attr_normal"), SFXLevel=ATTACK_SOUND_LEVEL_M,SFXType=COLLISION_SOUND_ATTR_FIRE, Type=ATTACK_REGION_ENERGY)
            });
        }
        if owner_module_accessor.status_kind() == *FIGHTER_STATUS_KIND_ATTACK_LW4{
            let effect_size = owner_module_accessor.get_float(FIGHTER_VEGETA_INSTANCE_WORK_ID_FLOAT_LW4_EFFECT_SIZE);
            let hitbox_size = owner_module_accessor.get_float(FIGHTER_VEGETA_INSTANCE_WORK_ID_FLOAT_LW4_HITBOX_SIZE);
            let damage = owner_module_accessor.get_float(FIGHTER_VEGETA_INSTANCE_WORK_ID_FLOAT_LW4_DAMAGE);
            if [15.0, 29.0].contains(&owner_module_accessor.motion_frame()){
                let bbatk = EffectModule::req_follow(weapon_module_accessor, Hash40::new("sys_killereye_bullet"), smash::phx::Hash40::new("top"), &ZERO_VECTOR, &Vector3f{x: 180.0, y: 0.0, z: 0.0}, effect_size, true, 0, 0, 0, 0, 0, true, true) as u32;
                EffectModule::set_rgb(weapon_module_accessor, bbatk, 0.0, 0.8, 1.0);
            }
            acmd!(lua_state, {
                ATTACK(ID=0, Part=0, Bone=hash40("top"), Damage=damage, Angle=40, KBG=60, FKB=0, BKB=50, Size=hitbox_size, X=0.0, Y=0.0, Z=0.0, X2=LUA_VOID, Y2=LUA_VOID, Z2=LUA_VOID, Hitlag=1.0, SDI=1.0, Clang_Rebound=ATTACK_SETOFF_KIND_ON, FacingRestrict=ATTACK_LR_CHECK_SPEED, SetWeight=false, ShieldDamage=-7, Trip=0.0, Rehit=0, Reflectable=true, Absorbable=true, Flinchless=false, DisableHitlag=false, Direct_Hitbox=false, Ground_or_Air=COLLISION_SITUATION_MASK_GA, Hitbits=COLLISION_CATEGORY_MASK_ALL, CollisionPart=COLLISION_PART_MASK_ALL, FriendlyFire=false, Effect=hash40("collision_attr_normal"), SFXLevel=ATTACK_SOUND_LEVEL_M,SFXType=COLLISION_SOUND_ATTR_FIRE, Type=ATTACK_REGION_ENERGY)
            });
        }

    }
}

fn update_effect_from_file() -> String{
    std::fs::read_to_string("sd:/effect.txt").unwrap()
}



unsafe fn print_all_fighters_motions(){
    for i in 0..FighterManager::entry_count(FIGHTER_MANAGER){
        let module_accessor = get_module_accessor_by_entry_id(i);
        println!("[print_all_fighter_motions] Fighter Num: {} Motion {:#x}", i, MotionModule::motion_kind(module_accessor));
    }
}



#[fighter_frame(agent = FIGHTER_KIND_LUCARIO)]
pub fn vegeta_frame(fighter : &mut L2CFighterCommon) {
    unsafe {
        let lua_state = fighter.lua_state_agent;
        let boma = smash::app::sv_system::battle_object_module_accessor(lua_state);
        let status_kind = boma.status_kind();
        let entry_id = get_entry_id(boma);
        if ControlModule::check_button_trigger(boma, *CONTROL_PAD_BUTTON_APPEAL_S_L){
            //print_all_fighters_motions();
        }
        if boma.is_status_one_of(&[*FIGHTER_STATUS_KIND_ATTACK_S4_HOLD, *FIGHTER_STATUS_KIND_ATTACK_S4, *FIGHTER_STATUS_KIND_ATTACK_S4_START]){
            if boma.is_infliction_status(*COLLISION_KIND_MASK_HIT){
                if !boma.is_flag(FIGHTER_VEGETA_INSTANCE_WORK_ID_FLAG_USED_S4_EFFECT){
                    EffectModule::req_on_joint(boma, Hash40::new("sys_togezoshell_bomb"), smash::phx::Hash40::new("haver"), &ZERO_VECTOR, &ZERO_VECTOR, 1.0, &ZERO_VECTOR, &ZERO_VECTOR, false, 0, 0, 0) as u32;
                    boma.on_flag(FIGHTER_VEGETA_INSTANCE_WORK_ID_FLAG_USED_S4_EFFECT);
                }
            }
        }
        else{
            boma.off_flag(FIGHTER_VEGETA_INSTANCE_WORK_ID_FLAG_USED_S4_EFFECT);
        }
        if boma.is_status(*FIGHTER_STATUS_KIND_ATTACK_LW4_HOLD){
            boma.inc_int(FIGHTER_VEGETA_INSTANCE_WORK_ID_INT_LW4_CHARGE_FRAME);
            let mut charge = boma.get_int(FIGHTER_VEGETA_INSTANCE_WORK_ID_INT_LW4_CHARGE_FRAME) as f32 / 120.0;
            let mut hitbox_size = (charge * 3.5) + 7.0;
            let mut effect_size = (charge * 0.75) + 1.5;
            let mut damage = (charge * 7.0) + 15.0;
            if charge > 1.0{
                charge = 1.0;
            }
            if hitbox_size > 10.5{
                hitbox_size = 10.5;
            }
            if effect_size > 2.25{
                effect_size = 2.25;
            }
            if damage > 22.0{
                damage = 22.0;
            }
            boma.set_float(hitbox_size, FIGHTER_VEGETA_INSTANCE_WORK_ID_FLOAT_LW4_HITBOX_SIZE);
            boma.set_float(effect_size, FIGHTER_VEGETA_INSTANCE_WORK_ID_FLOAT_LW4_EFFECT_SIZE);
            boma.set_float(damage, FIGHTER_VEGETA_INSTANCE_WORK_ID_FLOAT_LW4_DAMAGE);
            let hold_eff_size =  (charge) + 2.0;
            if !boma.is_flag(FIGHTER_VEGETA_INSTANCE_WORK_ID_FLAG_USED_LW4_EFFECT){
                let charge_1 = EffectModule::req_follow(boma, Hash40::new("sys_sscope_bullet_max"), smash::phx::Hash40::new("haver"), &Vector3f{ x: 0.0, y: 0.0, z: 5.0 }, &ZERO_VECTOR, hold_eff_size, true, 0, 0, 0, 0, 0, true, true) as u32;
                let charge_2 = EffectModule::req_follow(boma, Hash40::new("sys_sscope_bullet_max"), smash::phx::Hash40::new("havel"), &Vector3f{ x: 0.0, y: 0.0, z: 5.0 }, &ZERO_VECTOR, hold_eff_size, true, 0, 0, 0, 0, 0, true, true) as u32;
                EffectModule::set_rgb(boma, charge_1, 0.0, 0.8, 13.0);
                EffectModule::set_rgb(boma, charge_2, 0.0, 0.8, 13.0);
                boma.on_flag(FIGHTER_VEGETA_INSTANCE_WORK_ID_FLAG_USED_LW4_EFFECT);
                boma.set_int(charge_1 as i32, FIGHTER_VEGETA_INSTANCE_WORK_ID_INT_LW4_RIGHT_EFFECT_HANDLE);
                boma.set_int(charge_2 as i32, FIGHTER_VEGETA_INSTANCE_WORK_ID_INT_LW4_LEFT_EFFECT_HANDLE);
            }
        }
        else if boma.is_status(*FIGHTER_STATUS_KIND_ATTACK_LW4){
            if boma.motion_frame() == 14.0{
                EffectModule::kill(boma, boma.get_int(FIGHTER_VEGETA_INSTANCE_WORK_ID_INT_LW4_RIGHT_EFFECT_HANDLE) as u32, true, true);
            }
            if boma.motion_frame() == 28.0{
                EffectModule::kill(boma, boma.get_int(FIGHTER_VEGETA_INSTANCE_WORK_ID_INT_LW4_LEFT_EFFECT_HANDLE) as u32, true, true);
            }
        }
        if !boma.is_status_one_of(&[*FIGHTER_STATUS_KIND_ATTACK_LW4_HOLD, *FIGHTER_STATUS_KIND_ATTACK_LW4, *FIGHTER_STATUS_KIND_ATTACK_LW4_START, *FIGHTER_STATUS_KIND_SPECIAL_S]){
            boma.set_int(0, FIGHTER_VEGETA_INSTANCE_WORK_ID_INT_LW4_CHARGE_FRAME);
            boma.set_float(4.67, FIGHTER_VEGETA_INSTANCE_WORK_ID_FLOAT_LW4_HITBOX_SIZE);
            boma.set_float(1.0, FIGHTER_VEGETA_INSTANCE_WORK_ID_FLOAT_LW4_EFFECT_SIZE);
            boma.set_float(18.0, FIGHTER_VEGETA_INSTANCE_WORK_ID_FLOAT_LW4_DAMAGE);
            boma.off_flag(FIGHTER_VEGETA_INSTANCE_WORK_ID_FLAG_USED_LW4_EFFECT);
            EffectModule::kill_kind(boma, Hash40::new("sys_sscope_bullet_max"), true, true);
        }
        ModelModule::set_mesh_visibility(boma, Hash40::new("ken_earring_p"), true);
        if status_kind == *FIGHTER_STATUS_KIND_ATTACK_S3 && !boma.is_infliction_status(*COLLISION_KIND_MASK_HIT) && boma.motion_frame() > 20.0{
            boma.enable_cancel_into(CancelKind::Attack);
            boma.enable_cancel_into(CancelKind::Catch);
            boma.enable_cancel_into(CancelKind::SpecialN);
            boma.enable_cancel_into(CancelKind::SpecialS);
            boma.enable_cancel_into(CancelKind::SpecialHi);
            boma.enable_cancel_into(CancelKind::SpecialLw);
        }
        EffectModule::kill_kind(boma, Hash40::new("lucario_hadou"), true, true);
        EffectModule::kill_kind(boma, Hash40::new("lucario_hadou_l"), true, true);
        EffectModule::kill_kind(boma, Hash40::new("lucario_hadou_l_l"), true, true);
        EffectModule::kill_kind(boma, Hash40::new("lucario_hadou_l_r"), true, true);
        EffectModule::kill_kind(boma, Hash40::new("lucario_hadou_m"), true, true);

        GroundModule::set_rhombus_offset(boma, &smash::phx::Vector2f{
            x: 0.0,
            y: -1.0
        });
        if boma.is_button_on(Buttons::Special){
            if boma.is_button_trigger(Buttons::AppealSR){
                TEST2+=10;
                println!("{TEST2}");
            }
            if boma.is_button_trigger(Buttons::AppealSL){
                TEST2-=10;
                println!("{TEST2}");
            }
            if boma.is_button_trigger(Buttons::AppealHi){
                TEST2+=1;
                println!("{TEST2}");
            }
            if boma.is_button_trigger(Buttons::AppealLw){
                TEST2-=1;
                println!("{TEST2}");
            }
        }
       else if boma.is_button_on(Buttons::Attack){
            if boma.is_button_trigger(Buttons::AppealSR){
                TEST3+=10;
                println!("{TEST3}");
            }
            if boma.is_button_trigger(Buttons::AppealSL){
                TEST3-=10;
                println!("{TEST3}");
            }
            if boma.is_button_trigger(Buttons::AppealHi){
                TEST3+=1;
                println!("{TEST3}");
            }
            if boma.is_button_trigger(Buttons::AppealLw){
                TEST3-=1;
                println!("{TEST3}");
            }
        }
        else{
            if MotionModule::motion_kind(boma) == hash40("attack_air_lw"){
                enable_jump(boma);
            }
            if boma.is_button_trigger(Buttons::AppealSR){
                TEST+=10;
                println!("{TEST}");
            }
            if boma.is_button_trigger(Buttons::AppealSL){
                TEST-=10;
                println!("{TEST}");
            }
            if boma.is_button_trigger(Buttons::AppealHi){
                TEST+=1;
                println!("{TEST}");
            }
            if boma.is_button_trigger(Buttons::AppealLw){
                TEST-=1;
                println!("{TEST}");
            }
        }
        /*
        if ControlModule::check_button_on(module_accessor, *CONTROL_PAD_BUTTON_SPECIAL){
            if ControlModule::check_button_trigger(module_accessor, *CONTROL_PAD_BUTTON_APPEAL_S_L){
                b+=0.1;
                println!("{}", b);
            }
            if ControlModule::check_button_trigger(module_accessor, *CONTROL_PAD_BUTTON_APPEAL_S_R){
                b-=0.1;
                println!("{}", b);
            }
        }
        else{
            if ControlModule::check_button_trigger(module_accessor, *CONTROL_PAD_BUTTON_APPEAL_S_L){
                r+=0.1;
                println!("{}", r);
            }
            if ControlModule::check_button_trigger(module_accessor, *CONTROL_PAD_BUTTON_APPEAL_S_R){
                r-=0.1;
                println!("{}", r);
            }
        }
        if ControlModule::check_button_trigger(module_accessor, *CONTROL_PAD_BUTTON_APPEAL_HI){
            g+=0.1;
            println!("{}", g);
        }
        if ControlModule::check_button_trigger(module_accessor, *CONTROL_PAD_BUTTON_APPEAL_LW){
            g-=0.1;
            println!("{}", g);
        }

         */
        qcf_handle(boma);
        qcb_handle(boma);
        if ![FIGHTER_VEGETA_STATUS_KIND_GALICK_GUN_HOLD, FIGHTER_VEGETA_STATUS_KIND_GALICK_GUN_START, FIGHTER_VEGETA_STATUS_KIND_GALICK_GUN_FIRE].contains(&status_kind){
            CHARGE_TIME[entry_id] = 0.0;
            base_neutral_face(boma);
        }
        else{
            if [FIGHTER_VEGETA_STATUS_KIND_GALICK_GUN_START, FIGHTER_VEGETA_STATUS_KIND_GALICK_GUN_HOLD].contains(&status_kind){
                base_power_charge_face_n(boma);
            }
            else{
                base_power_attack_face_n(boma);
            }
        }
    }
}

pub fn install() {
    smashline::install_agent_frames!(vegeta_frame, hadoken);
}