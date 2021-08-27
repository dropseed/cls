use dialoguer::Confirm;
use dirs;
use std::env;
use std::error::Error;
use std::fs;
use std::path;
use uuid::Uuid;

const DEFAULT_REQUEST_PROMPT: &str = r#"
To help improve the quality of our tools, we track basic
anonymized usage information so we can learn what features
are used and how people use them.

Here's an example of an event we would collect:
{event_data}

Your settings will be saved here and can be changed at any time:
{settings_path}

Can we collect anonymous usage data from your installation?
"#;

#[derive(Debug)]
pub struct Settings {
    pub project_slug: String,
    pub request_permission_prompt: String,
    pub noninteractive_tracking_enabled: bool,
    _is_noninteractive: Option<bool>,
    _project_key: String,
    _debug: bool,
    _user_id: String,
    _invocation_id: String,
}

// should probably be configurable too
const CLS_ENV_PREFIX: &str = "CLS";

fn get_env_setting(name: &str) -> Option<String> {
    let env_key = format!("{}_{}", CLS_ENV_PREFIX, name);
    let env_val = env::var(env_key);
    if env_val.is_ok() {
        let env_val = env_val.unwrap();
        if env_val.len() > 0 {
            return Some(env_val.clone());
        }
    }
    None
}

impl Settings {
    pub fn new() -> Settings {
        Settings {
            project_slug: String::from(""),
            request_permission_prompt: String::from(DEFAULT_REQUEST_PROMPT),
            noninteractive_tracking_enabled: false,
            _is_noninteractive: None, // defaults to CI env var unless explicitly set
            _project_key: String::from(""),
            _debug: false,
            _user_id: String::from(""),
            _invocation_id: String::from(""),
        }
    }

    pub fn set_project_key(&mut self, key: &str) {
        self._project_key = key.to_string();
    }

    pub fn get_project_key(&self) -> String {
        get_env_setting("PROJECT_KEY").unwrap_or(self._project_key.clone())
    }

    pub fn set_debug(&mut self, debug: bool) {
        self._debug = debug;
    }

    pub fn get_debug(&self) -> bool {
        let env_val = get_env_setting("DEBUG").unwrap_or("".to_string());
        if env_val.len() > 0 {
            return env_val != "false" && env_val != "0";
        }
        return self._debug;
    }

    pub fn set_user_id(&mut self, user_id: &str) {
        self._user_id = user_id.to_string();
    }

    pub fn get_user_id(&self) -> String {
        let env_val = get_env_setting("USER_ID");
        if env_val.is_some() {
            return env_val.unwrap();
        }

        if self._user_id.len() > 0 {
            return self._user_id.clone();
        }

        match self.get_user_settings().get("user_id") {
            Some(user_id) => user_id.as_str().unwrap().to_string(),
            None => Uuid::new_v4().to_string(),
        }
    }

    pub fn set_invocation_id(&mut self, invocation_id: &str) {
        self._invocation_id = invocation_id.to_string();
    }

    pub fn get_invocation_id(&self) -> String {
        let env_val = get_env_setting("INVOCATION_ID");
        if env_val.is_some() {
            return env_val.unwrap();
        }

        if self._invocation_id.len() > 0 {
            return self._invocation_id.clone();
        }

        Uuid::new_v4().to_string()
    }

    pub fn set_is_noninteractive(&mut self, is_noninteractive: bool) {
        self._is_noninteractive = Some(is_noninteractive);
    }

    pub fn get_is_noninteractive(&self) -> bool {
        if self._is_noninteractive.is_some() {
            return self._is_noninteractive.unwrap();
        }

        let env_val = get_env_setting("NONINTERACTIVE").unwrap_or("".to_string());
        if env_val.len() > 0 {
            return env_val != "false" && env_val != "0";
        }

        if env::var("CI").is_ok() {
            return true;
        }

        return false;
    }

    fn get_config_dir(&self) -> path::PathBuf {
        let mut settings_path = dirs::config_dir().unwrap();
        settings_path.push(format!("{}_cls", self.project_slug));
        settings_path
    }
    pub fn get_cache_dir(&self) -> path::PathBuf {
        let mut cache_dir = dirs::cache_dir().unwrap();
        cache_dir.push(format!("{}_cls", self.project_slug));
        cache_dir
    }

    fn get_user_settings_path(&self) -> path::PathBuf {
        let mut settings_path = self.get_config_dir();
        settings_path.push("settings.json");
        settings_path
    }

    fn get_user_settings(&self) -> serde_json::Value {
        let settings_path = self.get_user_settings_path();
        if !settings_path.exists() {
            return serde_json::Value::default();
        }
        let mut settings_file = fs::File::open(settings_path).unwrap();
        let settings = serde_json::from_reader(&mut settings_file).unwrap();
        return settings;
    }

    fn set_user_setting(&self, key: &str, value: &serde_json::Value) {
        let mut settings = self.get_user_settings();
        settings[key] = value.clone();
        let settings_path = self.get_user_settings_path();
        let settings_dir = settings_path.parent().unwrap();
        if !settings_dir.exists() {
            fs::create_dir_all(settings_dir).unwrap();
        }
        let mut settings_file = fs::File::create(settings_path).unwrap();
        serde_json::to_writer_pretty(&mut settings_file, &settings).unwrap();
    }

    pub fn should_track_event(
        &self,
        event_data: &serde_json::Value,
    ) -> Result<bool, Box<dyn Error>> {
        let user_settings = self.get_user_settings();

        if user_settings.get("user_id").is_none() {
            super::debug_print("No user_id found, generating a new unique one".to_string());
            self.set_user_setting(
                "user_id",
                &serde_json::to_value(&self.get_user_id()).unwrap(),
            );
        }

        if self.get_is_noninteractive() {
            return Ok(self.noninteractive_tracking_enabled);
        }

        let already_enabled = user_settings.get("tracking_enabled");
        if !already_enabled.is_none() {
            return Ok(already_enabled.unwrap().as_bool().unwrap());
        }

        let prompt = self.request_permission_prompt.trim();
        let prompt = prompt.replace(
            "{event_data}",
            &serde_json::to_string_pretty(event_data).unwrap(),
        );
        let prompt = prompt.replace(
            "{settings_path}",
            &self.get_user_settings_path().to_str().unwrap(),
        );
        let tracking_enabled = Confirm::new().with_prompt(prompt).interact()?;

        self.set_user_setting(
            "tracking_enabled",
            &serde_json::to_value(tracking_enabled).unwrap(),
        );
        return Ok(tracking_enabled);
    }
}