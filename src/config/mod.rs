use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub viber_api_key: Option<String>,
    pub admin_id: Option<String>,
    pub domain_root_url: Option<String>,
    pub dark_sky_api_key: Option<String>,
    pub hosting_root_url: Option<String>,
    pub database_url: Option<String>,
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
    pub google_maps_api_key: Option<String>
}

impl Config {
    pub fn get_config_dir(app_name: &str) -> PathBuf {
        let mut path_buf = dirs::home_dir().unwrap();
        path_buf.push(app_name);
        path_buf
    }

    #[cfg(debug_assertions)]
    pub fn read(app_name: &str) -> Config {
        Config::read_from_toml(app_name)
    }

    #[cfg(not(debug_assertions))]
    pub fn read(app_name: &str) -> Config {
        Config::read_from_env()
    }

    #[allow(dead_code)]
    fn read_from_env() -> Config {
        Config {
            admin_id: std::env::var("ADMIN_ID").ok(),
            viber_api_key: std::env::var("VIBER_API_KEY").ok(),
            dark_sky_api_key: std::env::var("DARK_SKY_API_KEY").ok(),
            domain_root_url: std::env::var("DOMAIN_ROOT_URL").ok(),
            hosting_root_url: std::env::var("HOSTING_ROOT_URL").ok(),
            database_url: std::env::var("DATABASE_URL").ok(),
            google_client_id: std::env::var("GOOGLE_CLIENT_ID").ok(),
            google_client_secret: std::env::var("GOOGLE_CLIENT_SECRET").ok(),
            google_maps_api_key: std::env::var("GOOGLE_MAPS_API_KEY").ok(),
        }
    }

    #[allow(dead_code)]
    fn read_from_toml(app_name: &str) -> Config {
        info!("Reading config");
        // Get the user's home dir path
        let mut path_buf = dirs::home_dir().unwrap();
        // append folder for storing user related files
        path_buf.push(app_name);
        std::fs::create_dir_all(&path_buf).expect("can't create dir");
        path_buf.push("config.toml");
        let toml_str = std::fs::read_to_string(path_buf).expect("No configration.");
        let decoded: Config =
            toml::from_str(toml_str.as_str()).expect("failed to parse config.toml");
        decoded
    }
}
