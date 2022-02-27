#[derive(Debug)]
pub struct SupportedDefinition {
    pub content_types: Vec<String>,
    pub name_contains: Vec<String>,
    pub major_version_required: i32,
}
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

pub fn supported_definition() -> SupportedDefinition {
    macos_supported_definition()
}

fn macos_supported_definition() -> SupportedDefinition {
    SupportedDefinition {
        name_contains: vec![
            "-macos-".to_string(),
            "-macos10.10-".to_string(),
            "-Darwin-".to_string(),
        ],
        content_types: vec!["application/gzip".to_string()],
        major_version_required: 3,
    }
}
