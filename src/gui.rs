use colored::Colorize;
use egui;

use crate::level::Level;
use regex::Regex;

pub struct Options {
    pub decimals: i32,
    pub info_path: String,
    pub checkpoints: String,
}

pub struct MaunaKea {
    options: Options,
    template: String,
    old_template: String,
    level: Level,
}

impl Default for MaunaKea {
    fn default() -> Self {
        Self {
            options: Options {
                decimals: 3,
                info_path: String::from(""),
                checkpoints: String::from("")
            },
            //TODO: deal with bubble columns and wind later
            //TODO: add custom water shit later, cant be bothered to rn
            template: "Pos: {Player.Position} Speed: {Player.Speed} \
            {CrystalStaticSpinner.Position}{DustStaticSpinner.Position}{FrostHelper.CustomSpinner@FrostTempleHelper.Position}\
            {VivHelper.Entities.CustomSpinner@VivHelper.Position}{Celeste.Mod.XaphanHelper.Entities.CustomSpinner@XaphanHelper.Position} ".to_owned() +
            
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
            old_template: String::from(""),
            level: Level::default()
        }
    }
}

impl eframe::App for MaunaKea {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::SidePanel::right("options")
            .resizable(false)
            .default_width(0.3)
            .show(&ctx, |ui| {
                ui.add(
                    egui::DragValue::new(&mut self.options.decimals)
                        .clamp_range(0..=3)
                        .prefix("Decimals: ")
                        .speed(0.05),
                );
                ui.label("test!");
            });
        egui::CentralPanel::default().show(&ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("infodump.txt path: ");
                ui.text_edit_singleline(&mut self.options.info_path);
            });
            ui.text_edit_multiline(&mut self.options.checkpoints);
            if ui.button("Set Custom Info Template").clicked() {
                let client = reqwest::blocking::Client::new();
                let resp1 = client.get("http://localhost:32270/tas/custominfo").send();
                let re = Regex::new(r"<pre>([\S\s]*)</pre>").unwrap();
                match &resp1 {
                    Ok(_v) => {
                        self.old_template = String::from(
                            re.captures(resp1.unwrap().text().unwrap().as_str())
                                .unwrap()
                                .get(1)
                                .unwrap()
                                .as_str(),
                        )
                    }
                    Err(_e) => println!("didnt work! :("),
                }
                let resp2 = client
                    .head(format!(
                        "http://localhost:32270/tas/custominfo?template={}",
                        self.template
                    ))
                    .send();
                match &resp2 {
                    Ok(_v) => println!(
                        "{}",
                        "Setting the Custom Info Template succeeded!".bright_green()
                    ),
                    Err(_e) => println!("didnt work! :("),
                }
            }
            if ui.button("Run (INCOMPLETE)").clicked() {
                println!("{}", "Running!".bright_green());
                self.level.load(self.options.info_path.clone());
                self.level.run_alg(self.options.checkpoints.clone());
            }
            ui.label("add more shit here later!");
        });
    }
}
