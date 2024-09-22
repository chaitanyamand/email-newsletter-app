use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application_port: u16,
}

#[derive(Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

pub fn get_configurations() -> Result<Settings, config::ConfigError> {
    let mut settings = config::Config::default();

    settings.merge(config::File::with_name("configurations"))?;

    settings.try_into()
}
