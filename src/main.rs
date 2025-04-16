use clap::Parser;
use indexmap::IndexMap;
use inquire::{Select, error::InquireError};
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, Write};
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

#[derive(Deserialize, Serialize)]
#[serde(transparent)]
struct OrderedJson(#[serde(with = "ordered_json_map")] IndexMap<String, Value>);

mod ordered_json_map {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(map: &IndexMap<String, Value>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        map.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<IndexMap<String, Value>, D::Error>
    where
        D: Deserializer<'de>,
    {
        IndexMap::<String, Value>::deserialize(deserializer)
    }
}

fn prompt_bump_type() -> String {
    let choices = vec!["major", "minor", "patch"];

    match Select::new("What would you like to bump?", choices.clone())
        .with_starting_cursor(2)
        .prompt()
    {
        Ok(choice) => choice.to_string(),
        Err(InquireError::OperationCanceled) | Err(InquireError::OperationInterrupted) => {
            // Exit cleanly on Ctrl+C or ESC
            std::process::exit(130); // 130 = standard exit code for SIGINT
        }
        Err(_) => {
            println!("What would you like to bump? [major|minor|patch] (default: patch): ");
            print!("> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let trimmed = input.trim();

            match trimmed {
                "major" | "minor" | "patch" => trimmed.to_string(),
                "" => "patch".to_string(),
                _ => {
                    eprintln!("Invalid input. Defaulting to patch.");
                    "patch".to_string()
                }
            }
        }
    }
}

fn main() {
    let args = Args::parse();

    let current_version = get_current_version().unwrap_or_else(|| {
        eprintln!("No version found in composer.json, package.json, or VERSION file.");
        exit(1);
    });

    let mut version = Version::parse(&current_version).expect("Invalid semantic version");
    println!("Current version: {}", version);

    let bump_type = args.bump.unwrap_or_else(prompt_bump_type);

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
fn update_json_version(path: &str, new_version: &str) {
    if !Path::new(path).exists() {
        return;
    }

    let contents = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };

    let mut ordered: OrderedJson = match serde_json::from_str(&contents) {
        Ok(v) => v,
        Err(_) => return,
    };

    let map = &mut ordered.0;
    let mut updated = false;

    if let Some(Value::String(version)) = map.get_mut("version") {
        *version = new_version.to_string();
        updated = true;
    }

    if updated {
        if let Ok(output) = serde_json::to_string_pretty(&ordered) {
            let _ = fs::write(path, output);
        }
    }
}

fn update_package_json(new_version: &str) {
    update_json_version("package.json", new_version);
}

fn update_composer_json(new_version: &str) {
    update_json_version("composer.json", new_version);
}

fn update_version_file(new_version: &str) {
    let path = "VERSION";
    if Path::new(path).exists() {
        let _ = fs::write(path, new_version);
    }
}
