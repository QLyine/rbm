use std::{path::PathBuf, collections::HashMap, hash::Hash};

use serde::{Deserialize, Serialize};
use serde_yaml;
use dirs;
use glob::glob;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(skip_deserializing,skip_serializing)]
    config_directory: PathBuf,
    #[serde(rename = "api-collection-directories")]
    api_collection_directory: Vec<PathBuf>,
    #[serde(rename = "global-context")]
    global_context: Option<HashMap<String, String>>,
}


pub type APIContext = HashMap<String, String>;

#[derive(Serialize, Deserialize, Debug)]
pub struct APIEndpoint {
    method: String,
    url: String,
    headers: Option<HashMap<String, String>>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct APIConfig {
    context: Option<HashMap<String, APIContext>>,
    endpoints: HashMap<String, APIEndpoint>
}

impl Config {
    fn set_config_path(&mut self, path: PathBuf) {
        self.config_directory = path;
    }

    fn read_api(api_file: &PathBuf) -> HashMap<String, APIConfig> {
        let error_msg = format!("Could not open api file {:?}", api_file);
        let file_reader = std::fs::File::open(api_file).expect(error_msg.as_str());
        return serde_yaml::from_reader(file_reader).expect("Could not parse config file");
    }

    pub fn read_apis(&self) -> HashMap<String, APIConfig> {
        let mut result: HashMap<String, APIConfig> = HashMap::new();

        for api_directory in self.api_collection_directory.iter() {
            let mut abs_api_dir = if ! api_directory.is_absolute() {
                let mut abs_dir = self.config_directory.clone();
                abs_dir.push(api_directory);
                abs_dir
            } else {
                api_directory.clone()
            };
            if ! abs_api_dir.is_dir() {
                println!("Configured api collection directory: {:?} is not a folder", abs_api_dir.as_os_str());
                std::process::exit(1)
            }
            if ! abs_api_dir.exists() {
                continue;
            }
            abs_api_dir.push("*.y*ml");
            for maybe_file in glob(abs_api_dir.to_str().unwrap()).expect("Error listing files") {
                match maybe_file {
                    Ok(file) => {
                        let apis = Config::read_api(&file);
                        result.extend(apis)
                    }
                    Err(e) => {}
                }
            }
        }
        return result;
    }
}

pub fn read_config_or_create_default(maybe_config: &Option<PathBuf>) -> Config {
    return match maybe_config {
        Some(file) => read_config(file),
        None => read_default_config()
    }
}

fn default_rbm_directory() -> PathBuf {
    let mut config_directory = dirs::config_dir().expect("Could not read the default config directory");
    config_directory.push("rbm");
    return config_directory;
}

fn create_default_config() -> Config {
    let apis: Vec<PathBuf> = vec!["apis".into()];
    return Config { config_directory: default_rbm_directory(), api_collection_directory: apis, global_context: None };
}

fn write_config(path: PathBuf, config: &Config) {
    let file_writer = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(path)
        .expect("Could not write into config file");
    serde_yaml::to_writer(file_writer, config).expect("Could not write into config file")
}

fn read_default_config() -> Config {
    let config_directory = default_rbm_directory();
    if ! config_directory.exists() {
        std::fs::create_dir(&config_directory).unwrap();
    }
    let mut config_file_path: PathBuf = config_directory.clone();
    config_file_path.push("config");
    if config_file_path.exists() {
        return read_config(&config_file_path)
    }
    let default_config = create_default_config();
    write_config(config_file_path, &default_config);
    return default_config;
}

fn read_config(file_config: &PathBuf) -> Config {
    let file_reader = std::fs::File::open(file_config).expect("Could not open config file");
    let mut config: Config = serde_yaml::from_reader(file_reader).expect("Could not parse config file");
    let mut file_directory = std::fs::canonicalize(file_config).unwrap();
    //let mut file_directory = file_config.clone(); 
    file_directory.pop();
    config.set_config_path(file_directory);
    return config
}