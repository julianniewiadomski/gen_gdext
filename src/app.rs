use crate::utils::*;
use eframe::egui::{self};
use std::fs;
use std::sync::{Arc, Mutex};
use std::thread;

const DEFAULT_GODOT_VERSION: &str = "4.2";
const PROJECT_NAME_HINT: &str = "Logs will appear here...";
const LOG_MAX_HEIGHT: f32 = 300.0;
const LOG_TEXT_WIDTH: f32 = 470.0;

pub struct App {
    project_name: String,
    log: Arc<Mutex<String>>,
    is_creating: bool,
    templates: Option<ProjectTemplates>,
    godot_version: String,
    reloadable: bool,
    targets: Vec<(String, bool)>,
    autofocus_input: bool,
    precompile_lib: bool,
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self {
            godot_version: DEFAULT_GODOT_VERSION.to_string(),
            reloadable: true,
            targets: vec![
                ("linux.debug.x86_64".to_string(), true),
                ("linux.release.x86_64".to_string(), true),
                ("windows.debug.x86_64".to_string(), true),
                ("windows.release.x86_64".to_string(), true),
                ("macos.debug".to_string(), true),
                ("macos.release".to_string(), true),
            ],
            log: Arc::new(Mutex::new(String::new())),
            is_creating: false,
            templates: None,
            project_name: String::new(),
            autofocus_input: true,
            precompile_lib: false,
        };
        app.load_templates();
        app
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let is_creating = Arc::new(Mutex::new(false));
        self.is_creating = *is_creating.lock().unwrap();

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.show_project_name(ui);
                if !self.is_creating {
                    if ui.button("Create Project").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(egui::WindowLevel::AlwaysOnTop));
                        self.is_creating = true;
                        let log_clone = Arc::clone(&self.log);
                        let project_name = self.project_name.clone();
                        let templates = self.templates.clone();
                        let godot_version = self.godot_version.clone();
                        let reloadable = self.reloadable;
                        let targets: Vec<String> = self
                            .targets
                            .iter()
                            .filter_map(|(target, is_selected)| if *is_selected { Some(target.clone()) } else { None })
                            .collect();

                        // Spawn a new thread for project creation
                        let log_clone_inner = Arc::clone(&log_clone);
                        let log_clone_inner_clone = Arc::clone(&log_clone_inner);
                        let precompile_lib = self.precompile_lib;

                        thread::spawn(move || {
                            let result = handle_create_project(
                                &project_name,
                                log_clone_inner_clone,
                                templates.as_ref(),
                                &godot_version,
                                reloadable,
                                &targets,
                                precompile_lib,
                            );

                            let mut log_inner = log_clone_inner.lock().unwrap();
                            if let Err(err) = result {
                                log_inner.push_str(&format!("Error: {}\n", err))
                            }
                        });
                    }
                } else {
                    show_creation_progress(ui);
                }
            });

            self.show_godot_version(ui);
            self.show_reloadable_checkbox(ui);
            self.show_targets_group(ui);
            ui.checkbox(&mut self.precompile_lib, "Precompile Rust Library and GdExtension (this takes a while)");
            self.show_log(ui);
        });

        ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(egui::WindowLevel::Normal));
        ctx.request_repaint(); // Request UI to repaint to reflect log changes
    }
}

impl App {
    fn load_templates(&mut self) {
        const TEMPLATE_FILE: &str = "templates.yaml";
        if let Ok(content) = std::fs::read_to_string(TEMPLATE_FILE) {
            self.templates = serde_yaml::from_str::<ProjectTemplates>(&content).ok();
        } else {
            eprintln!("Failed to load templates.");
        }
    }

    fn show_project_name(&mut self, ui: &mut egui::Ui) {
        ui.label("Project Name:");
        let pn = ui.text_edit_singleline(&mut self.project_name);
        if self.autofocus_input {
            pn.request_focus();
            self.autofocus_input = false;
        }
    }

    fn show_godot_version(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Godot Version:");
            ui.text_edit_singleline(&mut self.godot_version);
        });
    }

    fn show_reloadable_checkbox(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.reloadable, "Reloadable");
    }

    fn show_targets_group(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label("Targets:");
            for (target, is_selected) in &mut self.targets {
                ui.checkbox(is_selected, target.clone());
            }
        });
    }

    fn show_log(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label("Log:");
            egui::ScrollArea::vertical().max_height(LOG_MAX_HEIGHT).show(ui, |ui| {
                let mut log_content = self.log.lock().unwrap();
                ui.add_sized(
                    egui::vec2(LOG_TEXT_WIDTH, LOG_MAX_HEIGHT),
                    egui::TextEdit::multiline(&mut *log_content)
                        .desired_rows(10)
                        .hint_text(PROJECT_NAME_HINT)
                        .interactive(false),
                );
            });
        });
    }
}

fn handle_create_project(
    project_name: &str,
    log_clone: Arc<Mutex<String>>,
    templates: Option<&ProjectTemplates>,
    godot_version: &str,
    reloadable: bool,
    targets: &[String],
    precompile_lib: bool,
) -> Result<(), String> {
    if project_name.is_empty() {
        return Err("Project name cannot be empty.".to_string());
    }

    if project_exists(project_name) {
        return Err("Project with this name already exists.".to_string());
    }

    {
        let mut log_inner = log_clone.lock().unwrap();
        log_inner.push_str("Creating project...\n");
    }

    // Call the actual function to create the project
    let templates = match templates {
        Some(templates) => templates,
        None => return Err("Templates are not available.".to_string()),
    };

    create_project(project_name, log_clone, templates, godot_version, reloadable, targets, precompile_lib)?;

    Ok(())
}

fn show_creation_progress(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spinner();
    });
}

fn project_exists(project_name: &str) -> bool {
    fs::metadata(project_name).is_ok()
}
