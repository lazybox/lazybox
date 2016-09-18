extern crate lazybox_settings as settings;

use settings::Settings;

fn main() {
    let mut s = Settings::new("examples/defaults.yml").unwrap();
    s.override_with("examples/overrides.yml").unwrap();
    println!("{:#?}", s);
}
