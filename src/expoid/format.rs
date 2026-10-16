use serde::{Deserialize, Serialize};
use serde_json::Result;
use wgpu::Texture;

pub struct ExpieData {
    pub name: String,
    pub weight: f32,
    pub height: f32,
    pub blood: f32,

    pub textures: ExpieTextures,
}

impl ExpieData {
    pub fn new() -> ExpieData {
        let json_data = std::fs::read_to_string("./assets/expies/expie/data.json").expect("expie json not found");
        let data: ExpieJsonData = serde_json::from_str(&json_data).expect("json parse err");

        let name = data.name;
        let weight = match data.weight.len() {
            2 => rand::random_range(data.weight[0]..data.weight[1]),
            _ => 50.0,
        };

        let height = match data.height.len() {
            2 => rand::random_range(data.height[0]..data.height[1]),
            _ => 145.0,
        };

        let blood = data.blood;

        todo!()

    }
}

pub struct ExpieTextures {
    pub head: HeadTextures,
    pub eyes: EyeTextures
}

/// open, closed, back
pub enum EyeTextures {
    Normal(Texture, Texture, Texture),
    Sad(Texture, Texture, Texture),
    Sleep(Texture, Texture, Texture),
    Tired(Texture, Texture, Texture),
    Scared(Texture, Texture, Texture),
    Panic(Texture, Texture, Texture),
    Happy(Texture, Texture, Texture),
    Gone(Texture),
    GoneHealed(Texture),
}

pub enum HeadTextures {
    Default(Texture),
    Open(Texture),
    OpenSmall(Texture),
    Disfigured(Vec<Texture>),
}

#[derive(Deserialize, Debug)]
pub struct ExpieJsonData {
    pub name: String,
    pub weight: Vec<f32>,
    pub height: Vec<f32>,
    pub blood: f32,

    pub eyes: ExpieEyes,

    pub head: ExpieHead,
}

#[derive(Deserialize, Debug)]
struct ExpieEyes {
    pub normal: Vec<String>,
    pub sad: Vec<String>,
    pub sleep: Vec<String>,
    pub tired: Vec<String>,
    pub scared: Vec<String>,
    pub panic: Vec<String>,
    pub happy: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct ExpieHead {
    pub default: String,
    pub open: String,
    pub opensmall: String,
    pub disfigured: Vec<String>,
}