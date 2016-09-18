extern crate lazybox_settings as settings;

use settings::Settings;

fn main() {
    let mut s = Settings::new("examples/defaults.yml").unwrap();
    s.override_with("examples/overrides.yml").unwrap();
    println!("{:#?}", s);

    assert!(s["game"]["time_step"].as_f64().is_some());

    let g = &s["graphics"];
    assert!(g["fps_cap"].as_i64() == Some(60));
    assert!(g["accelerate"].as_bool() == Some(true));
    let e = &g["effects"];
    assert!(e[0].as_str() == Some("bloom"));
    assert!(e[1].as_str() == Some("fxaa"));

    let a = &s["audio"];
    assert!(a["high_quality"].as_bool() == Some(false));
    assert!(a["time_limit"].as_i64() == None);
    assert!(!s["inputs"]["stuff"][0].is_valid());
}
