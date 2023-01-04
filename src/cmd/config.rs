use std::{
    collections::HashMap,
    fmt::{self},
    marker::PhantomData,
    path::PathBuf,
    str::FromStr,
};

use dirs;
use glob::glob;
use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use serde_yaml;
use void::Void;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(skip_deserializing, skip_serializing)]
    config_directory: PathBuf,
    #[serde(rename = "api-collection-directories")]
    api_collection_directory: Vec<PathBuf>,
    #[serde(rename = "global-context")]
    global_context: Option<HashMap<String, String>>,
}

pub type APIContext = HashMap<String, String>;

#[derive(Serialize, Deserialize, Debug)]
pub struct APIEndpoint {
    pub method: APIMethod,
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
    #[serde(deserialize_with = "string_or_struct_opt", default)]
    pub body: Option<APIBody>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct APIBody {
    #[serde(rename = "type")]
    pub api_body_type: APIBodyType,
    #[serde(rename = "content")]
    pub content: String,
}

impl APIBody {
    pub fn new(content: &str, api_type: APIBodyType) -> APIBody {
        APIBody {
            api_body_type: api_type,
            content: content.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum APIBodyType {
    #[serde(alias = "file")]
    FILE,
    #[serde(alias = "string")]
    STRING,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum APIMethod {
    GET,
    POST,
    DELETE,
    PATCH,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct APIConfig {
    context: Option<HashMap<String, APIContext>>,
    endpoints: HashMap<String, APIEndpoint>,
}

impl APIConfig {
    pub fn new(
        context: Option<HashMap<String, APIContext>>,
        endpoints: HashMap<String, APIEndpoint>,
    ) -> APIConfig {
        APIConfig {
            context: context,
            endpoints: endpoints,
        }
    }
}

impl APIConfig {
    pub fn contains_endpoint(&self, endpoint: &str) -> bool {
        return self.endpoints.contains_key(endpoint);
    }
    pub fn get_api_endpoint(&self, endpoint: &str) -> Option<&APIEndpoint> {
        return self.endpoints.get(endpoint);
    }
    pub fn contains_context(&self, context: &str) -> bool {
        return self
            .context
            .as_ref()
            .map(|c| c.contains_key(context))
            .unwrap_or(false);
    }
    pub fn get_api_context(&self, context: &str) -> Option<&APIContext> {
        return self.context.as_ref().map(|c| c.get(context)).flatten();
    }
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
            let mut abs_api_dir = if !api_directory.is_absolute() {
                let mut abs_dir = self.config_directory.clone();
                abs_dir.push(api_directory);
                abs_dir
            } else {
                api_directory.clone()
            };
            if !abs_api_dir.is_dir() {
                println!(
                    "Configured api collection directory: {:?} is not a folder",
                    abs_api_dir.as_os_str()
                );
                std::process::exit(1)
            }
            if !abs_api_dir.exists() {
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
        None => read_default_config(),
    };
}

fn default_rbm_directory() -> PathBuf {
    let mut config_directory =
        dirs::config_dir().expect("Could not read the default config directory");
    config_directory.push("rbm");
    return config_directory;
}

fn create_default_config() -> Config {
    let apis: Vec<PathBuf> = vec!["apis".into()];
    return Config {
        config_directory: default_rbm_directory(),
        api_collection_directory: apis,
        global_context: None,
    };
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
    if !config_directory.exists() {
        std::fs::create_dir(&config_directory).unwrap();
    }
    let mut config_file_path: PathBuf = config_directory.clone();
    config_file_path.push("config");
    if config_file_path.exists() {
        return read_config(&config_file_path);
    }
    let default_config = create_default_config();
    write_config(config_file_path, &default_config);
    return default_config;
}

fn read_config(file_config: &PathBuf) -> Config {
    let file_reader = std::fs::File::open(file_config).expect("Could not open config file");
    let mut config: Config =
        serde_yaml::from_reader(file_reader).expect("Could not parse config file");
    let mut file_directory = std::fs::canonicalize(file_config).unwrap();
    file_directory.pop();
    config.set_config_path(file_directory);
    return config;
}

fn string_or_struct_opt<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = Void>,
    D: Deserializer<'de>,
{
    struct StringOrStructOpt<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for StringOrStructOpt<T>
    where
        T: Deserialize<'de> + FromStr<Err = Void>,
    {
        type Value = Option<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("comma-separated string or null")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: Deserializer<'de>,
        {
            string_or_struct(deserializer).map(Some)
        }
    }

    deserializer.deserialize_option(StringOrStructOpt(PhantomData))
}

impl FromStr for APIBody {
    // This implementation of `from_str` can never fail, so use the impossible
    // `Void` type as the error type.
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Result::Ok(APIBody {
            api_body_type: APIBodyType::STRING,
            content: s.to_string(),
        })
    }
}

fn string_or_struct<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr<Err = Void>,
    D: Deserializer<'de>,
{
    struct StringOrStruct<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + FromStr<Err = Void>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
        {
            Ok(FromStr::from_str(value).unwrap())
        }

        fn visit_map<M>(self, map: M) -> Result<T, M::Error>
        where
            M: MapAccess<'de>,
        {
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }

    deserializer.deserialize_any(StringOrStruct(PhantomData))
}
