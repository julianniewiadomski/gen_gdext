use serde::Deserialize;
use std::fs;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Deserialize, Clone)]
pub struct ProjectTemplates {
    pub gitignore: String,
    pub lib_content: String,
    pub gdextension: String,
    pub cargo_toml: String,
}

pub fn get_gitignore_content(templates: &ProjectTemplates) -> String {
    templates.gitignore.clone()
}

pub fn convert_to_camel_case(input: &str) -> String {
    input
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<String>()
}

pub fn get_lib_content(templates: &ProjectTemplates, project_name: &str) -> String {
    templates.lib_content.replace("{project_name}", &convert_to_camel_case(project_name))
}

pub fn get_gdextension_content(templates: &ProjectTemplates, project_name: &str, godot_version: &str, reloadable: bool, targets: &[String]) -> String {
    let mut content = templates
        .gdextension
        .replace("{project_name}", project_name)
        .replace("compatibility_minimum = 4.2", &format!("compatibility_minimum = {}", godot_version))
        .replace("reloadable = true", &format!("reloadable = {}", if reloadable { "true" } else { "false" }));

    let target_lines: Vec<String> = targets
        .iter()
        .filter_map(|target| {
            let library_path = match target.as_str() {
                "linux.debug.x86_64" => format!("res://rust/target/debug/lib{}.so", project_name),
                "linux.release.x86_64" => format!("res://rust/target/release/lib{}.so", project_name),
                "windows.debug.x86_64" => format!("res://rust/target/debug/{}.dll", project_name),
                "windows.release.x86_64" => format!("res://rust/target/release/{}.dll", project_name),
                "macos.debug" => format!("res://rust/target/debug/lib{}.dylib", project_name),
                "macos.release" => format!("res://rust/target/release/lib{}.dylib", project_name),
                _ => return None,
            };
            Some(format!("{} = \"{}\"", target, library_path))
        })
        .collect();

    if !target_lines.is_empty() {
        content.push_str(&format!("[libraries]\n{}\n", target_lines.join("\n")));
    } else {
        content.push_str("[libraries]\n"); // Optional: empty libraries section
    }

    content
}

pub fn create_project(
    project_name: &str,
    log: Arc<Mutex<String>>, // Change to Arc<Mutex<String>>
    templates: &ProjectTemplates,
    godot_version: &str,
    reloadable: bool,
    targets: &[String],
    precompile_lib: bool,
) -> Result<(), String> {
    let mut log_content = String::new();
    log_content.push_str(&format!("Creating project '{}'\n", project_name));

    // Create Godot project directory
    let godot_dir = project_name.to_string();
    fs::create_dir_all(&godot_dir).expect("Failed to create Godot project directory");

    // Create project.godot file
    let project_godot_content = format!("[gd_project]\nversion=4.0\nrun/main_scene=\"res://main.tscn\"\n");
    fs::write(format!("{}/project.godot", godot_dir), project_godot_content).expect("Failed to create project.godot file");

    // Create Rust folder inside the Godot project directory
    let rust_dir = format!("{}/rust", godot_dir);
    fs::create_dir_all(&rust_dir).expect("Failed to create Rust directory");

    // Write the Cargo.toml file
    let cargo_toml_content = templates.cargo_toml.replace("{project_name}", project_name);
    fs::write(format!("{}/Cargo.toml", rust_dir), cargo_toml_content).expect("Failed to create Cargo.toml file");

    // Write the .gitignore file
    let gitignore_content = get_gitignore_content(templates);
    fs::write(format!("{}/.gitignore", rust_dir), gitignore_content).expect("Failed to create .gitignore file");

    // Create Rust source directory
    let rust_src_dir = format!("{}/src", rust_dir);
    fs::create_dir_all(&rust_src_dir).expect("Failed to create Rust source directory");

    // Write lib.rs file
    let lib_content = get_lib_content(templates, project_name);
    fs::write(format!("{}/lib.rs", rust_src_dir), lib_content).expect("Failed to create lib.rs file");

    // Write .gdextension file
    let gdextension_content = get_gdextension_content(templates, project_name, godot_version, reloadable, targets);
    fs::write(format!("{}/{}.gdextension", godot_dir, project_name), gdextension_content).expect("Failed to create .gdextension file");

    {
        let mut log_inner = log.lock().unwrap();
        log_inner.push_str(&format!("Created Godot project '{}' with Rust integration.\n", project_name));
    }

    if precompile_lib {
        let log_clone = Arc::clone(&log);
        let project_name = project_name.to_string();
        let are_targets_empty = targets.is_empty();

        thread::spawn(move || {
            {
                let mut log_inner = log_clone.lock().unwrap();
                log_inner.push_str("Compiling Rust library...\n");
            }

            let lib_path = format!("{}/rust/src/lib.rs", project_name);

            if fs::metadata(lib_path).is_ok() && !are_targets_empty {
                let mut result = Command::new("cargo")
                    .arg("build")
                    .current_dir(format!("{}/rust/src", project_name))
                    .spawn()
                    .expect("Failed to start cargo build process");

                if result.wait().unwrap().success() {
                    {
                        let mut log_inner = log_clone.lock().unwrap();
                        log_inner.push_str("Rust library compiled successfully.\nProject created successfully.\n");
                    }
                } else {
                    let mut log_inner = log_clone.lock().unwrap();
                    log_inner.push_str("Failed to compile Rust library.\n");
                }
            } else {
                let mut log_inner = log_clone.lock().unwrap();
                log_inner.push_str("Rust library file does not exist.\n");
            }
        });
    } else {
        let mut log_inner = log.lock().unwrap();
        log_inner.push_str("Project created successfully.\n");
    }

    Ok(())
}
