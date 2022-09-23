use egui;

use crate::globals::OPTS;
use crate::level::Level;

pub struct MaunaKea {
    template:String,
    level:Level
}

impl Default for MaunaKea {
    fn default() -> Self {
        Self {
            //TODO: deal with bubble columns and wind later
            //TODO: add custom water shit later, cant be bothered to rn
            template: "PosRemainder: {Player.PositionRemainder} ".to_owned() +
            "Lerp: {Player.starFlySpeedLerp} " +
        
            "{CrystalStaticSpinner.Position}{DustStaticSpinner.Position}{FrostHelper.CustomSpinner@FrostTempleHelper.Position}{VivHelper.Entities.CustomSpinner@VivHelper.Position}{Celeste.Mod.XaphanHelper.Entities.CustomSpinner@XaphanHelper.Position} " +
            
            "LightningUL: {Lightning.TopLeft} " +
            "LightningDR: {Lightning.BottomRight} " +
        
            "SpikeUL: {Spikes.TopLeft} " +
            "SpikeDR: {Spikes.BottomRight} " +
            "SpikeDir: {Spikes.Direction} " +
        
            /*"Wind: {Level.Wind} " +
            "WTPos: {WindTrigger.Position} " +
            "WTPattern: {WindTrigger.Pattern} " +
            "WTWidth: {WindTrigger.Width} " +
            "WTHeight: {WindTrigger.Height} " +
        
            "add bubble column shit here " +*/
        
            "JThruUL: {JumpthruPlatform.TopLeft} " +
            "JThruDR: {JumpthruPlatform.BottomRight} " +
            "SideJTUL: {SidewaysJumpThru.TopLeft} " +
            "SideJTDR: {SidewaysJumpThru.BottomRight} " +
            "SideJTIsRight: {SidewaysJumpThru.AllowLeftToRight} " +
            "SideJTPushes: {SidewaysJumpThru.pushPlayer} " +
            "UpsDJTUL: {UpsideDownJumpThru.TopLeft} " +
            "UpsDJTDR: {UpsideDownJumpThru.BottomRight} " +
            "UpsDJTPushes: {UpsideDownJumpThru.pushPlayer} " +

            "WaterUL: {Water.TopLeft} " +
            "WaterDR: {Water.BottomRight} " +
        
            "Bounds: {Level.Bounds} " +
            "Solids: {Level.Session.LevelData.Solids}",
            level: Level::default()
        }
    }
}

impl eframe::App for MaunaKea {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("options").resizable(false).default_width(0.3).show(&ctx, |ui| {
            unsafe {
                ui.add(egui::DragValue::new(&mut OPTS.decimals).clamp_range(0..=3).prefix("Decimals: ").speed(0.05));
            }
            ui.label("test!");
        });
        egui::CentralPanel::default().show(&ctx, |ui| {
            ui.label("add shit here later!");
        });
    }
}