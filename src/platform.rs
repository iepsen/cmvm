use regex::Regex;
use std::process::Command;

#[derive(Debug)]
pub struct PlatformInfo {
    pub version: OperatingSystemVersion,
    pub name: String,
}
#[derive(Debug)]
pub struct OperatingSystemVersion {
    pub major: i32,
    pub minor: i32,
    pub patch: i32,
}

pub fn is_supported_platform() -> bool {
    std::env::consts::OS == "macos"
}

pub fn get_platform_info() -> Result<PlatformInfo, Box<dyn std::error::Error>> {
    let output = Command::new("sw_vers").output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let version = parse_version(stdout.to_string())?;

    // macOS 10.13 or later == macos
    // macOS 10.10 or later == macos10.10
    let mut name = "macos".to_string();
    if version.major == 10 && version.minor >= 10 {
        name = "macos10.10".to_string();
    }
    Ok(PlatformInfo { name, version })
}

fn extract_from_regex(stdout: &String, regex: Regex) -> Option<String> {
    match regex.captures_iter(&stdout).next() {
        Some(m) => match m.get(1) {
            Some(s) => Some(s.as_str().to_owned()),
            None => None,
        },
        None => None,
    }
}

pub fn parse_version(
    version_str: String,
) -> Result<OperatingSystemVersion, Box<dyn std::error::Error>> {
    let regex = Regex::new(r"ProductVersion:\s(\w+\.\w+\.\w+)").unwrap();

    let system_version = match extract_from_regex(&version_str, regex) {
        Some(system_version) => system_version,
        None => "".to_string(),
    };

    let mut system_version = system_version.splitn(3, ".");
    let (major, minor, patch) = (
        system_version.next().unwrap().parse::<i32>().unwrap(),
        system_version.next().unwrap().parse::<i32>().unwrap(),
        system_version.next().unwrap().parse::<i32>().unwrap(),
    );

    Ok(OperatingSystemVersion {
        major,
        minor,
        patch,
    })
}
