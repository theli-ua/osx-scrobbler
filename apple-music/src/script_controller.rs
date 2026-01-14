use crate::error::Error;
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::process::{Command, Output};
use strum_macros::Display;

#[derive(Serialize, Display)]
#[strum(serialize_all = "camelCase")]
pub enum ParamType {
    Artworks,
    AllTracks,
    CurrentTrack,
    PlaylistById,
    PlaylistTracks,
    ApplicationData,
    SearchInPlaylist,
}

/// TEST
pub struct ScriptController;

impl ScriptController {
    pub fn execute_script<T>(
        &self,
        param_type: ParamType,
        id: Option<i32>,
        query: Option<&str>,
    ) -> Result<T, Error>
    where
        T: for<'a> Deserialize<'a>,
    {
        let params = self.generate_json(param_type, id, query);
        let script_path = include_str!("scripts/script.js");

        let output = self.execute(script_path, Some(params));

        let data;
        match output {
            Ok(d) => data = d,
            Err(err) => {
                error!("{:#?}", err);
                return Err(err);
            }
        }

        let output_str = String::from_utf8_lossy(&data.stdout);
        println!("Output: {}", output_str);

        return match serde_json::from_str::<T>(&output_str) {
            Ok(data) => Ok(data),
            Err(err) => {
                error!("{:#?}", err);
                Err(Error::DeserializationFailed)
            }
        };
    }

    pub fn execute(&self, command: &str, params: Option<String>) -> Result<Output, Error> {
        let mut binding = Command::new("osascript");
        let output = binding.arg("-l").arg("JavaScript").arg("-e").arg(command);

        if let Some(params) = params {
            output.arg(params);
        }
        let data;
        match output.output() {
            Ok(d) => data = d,
            Err(err) => {
                error!("{:#?}", err);
                return Err(Error::NoData);
            }
        }

        return Ok(data);
    }

    fn generate_json(&self, param_type: ParamType, id: Option<i32>, query: Option<&str>) -> String {
        let mut hmap = HashMap::<&str, String>::new();

        hmap.insert("param_type", param_type.to_string());

        if let Some(id) = id {
            hmap.insert("id", id.to_string());
        }

        if let Some(query) = query {
            hmap.insert("query", query.into());
        }
        return json!(hmap).to_string();
    }
}
