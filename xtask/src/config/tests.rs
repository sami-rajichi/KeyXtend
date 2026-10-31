//! Tests of the tidy and DCO config.

use std::collections::BTreeMap;

use super::*;
use crate::test_support::tidy_config as tidy;

#[test]
fn valid_config_passes_validation() {
    assert_eq!(tidy().validate(), Ok(()));
}

#[test]
fn warn_greater_than_max_is_rejected() {
    let mut config = tidy();
    config.warn_file_lines = config.max_file_lines + 1;
    assert_eq!(config.validate(), Err(ConfigError::WarnExceedsMax));
}

#[test]
fn warn_equal_to_max_is_allowed() {
    let mut config = tidy();
    config.warn_file_lines = config.max_file_lines;
    assert_eq!(config.validate(), Ok(()));
}

#[test]
fn each_required_list_must_not_be_empty() {
    let emptied = |name: &'static str, clear: fn(&mut TidyConfig)| {
        let mut config = tidy();
        clear(&mut config);
        assert_eq!(config.validate(), Err(ConfigError::EmptyList(name)));
    };
    emptied("extensions", |c| c.extensions.clear());
    emptied("scan_dirs", |c| c.scan_dirs.clear());
    emptied("spawn_dirs", |c| c.spawn_dirs.clear());
    emptied("spawn_extensions", |c| c.spawn_extensions.clear());
    emptied("banned_spawns", |c| c.banned_spawns.clear());
}

/// Each list of the tidy config, by name.
fn lists(c: &mut TidyConfig) -> [(&'static str, &mut Vec<String>); 12] {
    [
        ("extensions", &mut c.extensions),
        ("scan_dirs", &mut c.scan_dirs),
        ("skip_dirs", &mut c.skip_dirs),
        ("module_allowed", &mut c.module_allowed),
        ("banned_network", &mut c.banned_network),
        ("banned_media", &mut c.banned_media),
        ("root_kinds", &mut c.root_kinds),
        ("test_only", &mut c.test_only),
        ("spawn_dirs", &mut c.spawn_dirs),
        ("spawn_extensions", &mut c.spawn_extensions),
        ("banned_spawns", &mut c.banned_spawns),
        ("build_scripts", &mut c.build_scripts),
    ]
}

#[test]
fn a_blank_entry_in_any_list_is_rejected() {
    let count = lists(&mut tidy()).len();
    for (i, blank) in (0..count).flat_map(|i| [(i, ""), (i, "  ")]) {
        let mut config = tidy();
        let (name, list) = lists(&mut config).into_iter().nth(i).expect("a list");
        list.push(blank.to_string());
        assert_eq!(config.validate(), Err(ConfigError::BlankEntry(name)));
    }
}

/// Each feature map of the tidy config, by name.
fn maps(c: &mut TidyConfig) -> [(&'static str, &mut BTreeMap<String, Vec<String>>); 2] {
    [
        ("banned_features", &mut c.banned_features),
        ("build_only_features", &mut c.build_only_features),
    ]
}

#[test]
fn a_blank_crate_or_feature_in_a_feature_map_is_rejected() {
    let blanks: [(&str, &[&str]); 4] = [
        ("", &["x"]),
        (" ", &["x"]),
        ("kx-lib", &[" "]),
        ("kx-lib", &[]),
    ];
    for i in 0..maps(&mut tidy()).len() {
        for (n, (krate, features)) in blanks.iter().enumerate() {
            let mut config = tidy();
            let (name, map) = maps(&mut config).into_iter().nth(i).expect("a map");
            let features = features.iter().map(ToString::to_string).collect();
            map.insert((*krate).to_string(), features);
            let want = Err(ConfigError::BlankEntry(name));
            assert_eq!(config.validate(), want, "{name}, case {n}");
        }
    }
}

#[test]
fn a_build_script_outside_its_package_is_rejected() {
    for bad in ["../build.rs", "/build.rs", "C:/build.rs", "src/./build.rs"] {
        let mut config = tidy();
        config.build_scripts.push(bad.to_string());
        let want = Err(ConfigError::NotRelative("build_scripts"));
        assert_eq!(config.validate(), want, "{bad}");
    }
    let mut config = tidy();
    config.build_scripts.push("build/gen.rs".to_string());
    assert_eq!(config.validate(), Ok(()), "a script in a folder");
}

#[test]
fn empty_tool_dir_is_rejected() {
    let mut config = tidy();
    config.tool_dir = String::new();
    assert_eq!(config.validate(), Err(ConfigError::EmptyValue("tool_dir")));
}

#[test]
fn blank_deny_attr_is_rejected() {
    let mut config = tidy();
    config.deny_attr = "  ".to_string();
    assert_eq!(config.validate(), Err(ConfigError::EmptyValue("deny_attr")));
}

#[test]
fn blank_bridge_values_are_rejected() {
    let mut config = tidy();
    config.bridge_dir = String::new();
    assert_eq!(
        config.validate(),
        Err(ConfigError::EmptyValue("bridge_dir"))
    );
    let mut config = tidy();
    config.safety_tag = " ".to_string();
    assert_eq!(
        config.validate(),
        Err(ConfigError::EmptyValue("safety_tag"))
    );
    let mut config = tidy();
    config.bridge_crate = String::new();
    let want = Err(ConfigError::EmptyValue("bridge_crate"));
    assert_eq!(config.validate(), want);
}

#[test]
fn blank_build_tool_is_rejected() {
    let mut config = tidy();
    config.build_tool = " ".to_string();
    let want = Err(ConfigError::EmptyValue("build_tool"));
    assert_eq!(config.validate(), want);
}

#[test]
fn error_messages_name_their_values() {
    assert_eq!(
        ConfigError::ZeroValue("x").to_string(),
        "x must not be zero"
    );
    assert_eq!(
        ConfigError::NotPositive("x").to_string(),
        "x must be at least 1"
    );
    assert_eq!(
        ConfigError::NotQuotable("x").to_string(),
        "x must not contain \" or end with \\"
    );
    assert_eq!(
        ConfigError::NotBelow("a", "b").to_string(),
        "a must be smaller than b"
    );
    assert_eq!(
        ConfigError::Same("a", "b").to_string(),
        "a and b must differ"
    );
    assert_eq!(
        ConfigError::NotListed("a", "b").to_string(),
        "a must name only entries of b"
    );
    assert_eq!(
        ConfigError::NotType("a", "dll").to_string(),
        "a must hold only .dll files"
    );
    assert_eq!(
        ConfigError::BlankEntry("a").to_string(),
        "a must hold no blank entry"
    );
}

#[test]
fn tidy_config_rejects_unknown_field() {
    let json = r#"{
        "max_file_lines": 400, "warn_file_lines": 300,
        "extensions": [], "scan_dirs": [], "skip_dirs": [],
        "app_dir": "apps", "tool_dir": "tools", "build_tool": "xtask",
        "module_prefix": "kx-mod-", "platform_prefix": "kx-platform-",
        "module_allowed": [], "app": "keyxtend",
        "banned_network": [], "banned_media": [], "banned_features": {},
        "build_only_features": {}, "unsafe_attr": "x",
        "deny_attr": "y", "root_kinds": [], "test_only": [], "bridge_crate": "b",
        "bridge_dir": "d", "safety_tag": "t", "spawn_dirs": [], "spawn_extensions": [],
        "banned_spawns": [], "build_scripts": [], "bogus": 1
    }"#;
    assert!(serde_json::from_str::<TidyConfig>(json).is_err());
    let known = json.replace(r#", "bogus": 1"#, "");
    assert!(
        serde_json::from_str::<TidyConfig>(&known).is_ok(),
        "only the unknown field fails"
    );
}

#[test]
fn dco_config_rejects_unknown_field() {
    let json = r#"{"trailer": "Signed-off-by", "exempt_authors": [], "bogus": 1}"#;
    assert!(serde_json::from_str::<DcoConfig>(json).is_err());
}
