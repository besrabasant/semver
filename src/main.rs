use clap::Parser;
use dialoguer::Select;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, process::exit};

#[derive(Deserialize, Serialize)]
struct PackageJson {
    version: String,
    #[serde(flatten)]
    other: serde_json::Value,
}

#[derive(Deserialize, Serialize)]
struct ComposerJson {
    version: String,
    #[serde(flatten)]
    other: serde_json::Value,
}

/// CLI tool to bump semantic version
#[derive(Parser)]
#[command(author, version, about)]
struct Args {
    /// Optional version bump type: major, minor, patch
    #[arg(long)]
    bump: Option<String>,
}

fn main() {
    let args = Args::parse();

    let current_version = get_current_version().unwrap_or_else(|| {
        eprintln!("No version found in composer.json, package.json, or VERSION file.");
        exit(1);
    });

    let mut version = Version::parse(&current_version).expect("Invalid semantic version");
    println!("Current version: {}", version);

    let bump_type = match args.bump {
        Some(t) => t,
        None => {
            let choices = vec!["major", "minor", "patch"];
            let selection = Select::new()
                .with_prompt("What would you like to bump?")
                .items(&choices)
                .default(2)
                .interact()
                .unwrap();
            choices[selection].to_string()
        }
    };

    match bump_type.as_str() {
        "major" => {
            version.major += 1;
            version.minor = 0;
            version.patch = 0;
        }
        "minor" => {
            version.minor += 1;
            version.patch = 0;
        }
        "patch" => {
            version.patch += 1;
        }
        _ => {
            eprintln!("Invalid bump type: {}", bump_type);
            exit(1);
        }
    }

    version.pre = semver::Prerelease::EMPTY;
    version.build = semver::BuildMetadata::EMPTY;

    let new_version = version.to_string();
    println!("Bumping version {} → {}", current_version, new_version);

    update_package_json(&new_version);
    update_version_file(&new_version);
    update_composer_json(&new_version);
}

fn get_current_version() -> Option<String> {
    if Path::new("composer.json").exists() {
        if let Ok(contents) = fs::read_to_string("composer.json") {
            if let Ok(json) = serde_json::from_str::<ComposerJson>(&contents) {
                return Some(json.version);
            }
        }
    }

    if Path::new("package.json").exists() {
        if let Ok(contents) = fs::read_to_string("package.json") {
            if let Ok(json) = serde_json::from_str::<PackageJson>(&contents) {
                return Some(json.version);
            }
        }
    }

    if Path::new("VERSION").exists() {
        if let Ok(version) = fs::read_to_string("VERSION") {
            return Some(version.trim().to_string());
        }
    }

    None
}

fn update_package_json(new_version: &str) {
    let path = "package.json";
    if !Path::new(path).exists() {
        return;
    }

    if let Ok(contents) = fs::read_to_string(path) {
        let result = serde_json::from_str::<serde_json::Value>(&contents);
        if let Ok(mut json_value) = result {
            if let Some(version) = json_value.get_mut("version") {
                if version.is_string() {
                    *version = serde_json::Value::String(new_version.to_string());
                    if let Ok(updated) = serde_json::to_string_pretty(&json_value) {
                        let _ = fs::write(path, updated);
                    }
                }
            }
        }
    }
}


fn update_composer_json(new_version: &str) {
    let path = "composer.json";
    if !Path::new(path).exists() {
        return;
    }

    if let Ok(contents) = fs::read_to_string(path) {
        if let Ok(mut json) = serde_json::from_str::<ComposerJson>(&contents) {
            json.version = new_version.to_string();
            if let Ok(updated) = serde_json::to_string_pretty(&json) {
                let _ = fs::write(path, updated);
            }
        }
    }
}

fn update_version_file(new_version: &str) {
    let path = "VERSION";
    if Path::new(path).exists() {
        let _ = fs::write(path, new_version);
    }
}
