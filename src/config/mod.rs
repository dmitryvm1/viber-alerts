use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub viber_api_key: Option<String>,
    pub admin_id: Option<String>,
    pub domain_root_url: Option<String>,
    pub dark_sky_api_key: Option<String>
}

impl Config {
    pub fn get_config_dir(app_name: &str) -> PathBuf {
        let mut path_buf = dirs::home_dir().unwrap();
        path_buf.push(app_name);
        path_buf
    }

    pub fn read(app_name: &str) -> Config {
        println!("Reading config");
        // Get the user's home dir path
        let mut path_buf = dirs::home_dir().unwrap();
        // append folder for storing user related files
        path_buf.push(app_name);
        std::fs::create_dir_all(&path_buf).expect("can't create dir");
        path_buf.push("config.toml");
        let toml_str = std::fs::read_to_string(path_buf).expect("No configration.");
        let decoded: Config = toml::from_str(toml_str.as_str()).expect("failed to parse config.toml");
        decoded
    }
}