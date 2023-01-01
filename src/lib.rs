use image::GenericImageView;
use indexmap::IndexSet;
use regex::Regex;
use serde::Deserialize;
use std::{fs, path::PathBuf};

#[macro_use]
extern crate lazy_static;

#[derive(Deserialize, Clone, Debug)]
pub struct Token {
    pub uid: i32,
    pub token: String,
}

pub type Board = Vec<Vec<u8>>;

#[derive(Clone, Debug)]
pub struct Image {
    pub name: String,
    pub data: Board,
    pub x: usize,
    pub y: usize,
    pub len_x: usize,
    pub len_y: usize,
    pub priority: i32,
}

#[derive(Deserialize, Clone, Debug)]
pub struct DeserializeConfig {
    pub base_url: String,
    pub tokens: Vec<Token>,
    pub board_x: usize,
    pub board_y: usize,
    pub fetch_interval: u64,
    pub paint_interval: u64,
    pub max_concurrent_paint: usize,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub board_url: String,
    pub paint_url: String,
    pub tokens: Vec<Token>,
    pub images: Vec<Image>,
    pub board_x: usize,
    pub board_y: usize,
    pub fetch_interval: u64,
    pub paint_interval: u64,
    pub max_concurrent_paint: usize,
}

lazy_static! {
    static ref COLORS: IndexSet<(u8, u8, u8)> = {
        let mut ans = IndexSet::new();
        let re = Regex::new(r"(?P<r>\d*) (?P<g>\d*) (?P<b>\d*)").unwrap();

        for row in fs::read_to_string("./images/colors").unwrap().lines() {
            if let Some(caps) = re.captures(row) {
                ans.insert((
                    caps["r"].parse::<u8>().unwrap(),
                    caps["g"].parse::<u8>().unwrap(),
                    caps["b"].parse::<u8>().unwrap(),
                ));
            }
        }

        ans
    };
}

fn read_image(path: PathBuf, extension: String) -> Board {
    match extension.as_str() {
        "bmp" => {
            let img = image::open(path).unwrap();

            let width = img.width();
            let height = img.height();

            let mut ans = Vec::new();

            for i in 0..width {
                ans.push(Vec::new());

                for j in 0..height {
                    let [r, g, b, _] = img.get_pixel(i, j).0;
                    let color = COLORS.get_index_of(&(r, g, b)).unwrap() as u8;
                    ans[i as usize].push(color);
                }
            }

            ans
        }
        "txt" => {
            let data = fs::read_to_string(path).unwrap();
            let tmp: Board = data
                .split('\n')
                .map(|row| row.as_bytes().to_vec())
                .collect();

            let mut ans = Vec::new();

            for i in 0..tmp[0].len() - 1 {
                ans.push(Vec::new());

                tmp.iter()
                    .for_each(|row| ans[i].push(char_to_color(row[i])));
            }

            ans
        }
        other => panic!("invalid image extension: {}", other),
    }
}

lazy_static! {
    pub static ref CONFIG: Config = {
        let data = fs::read_to_string("./config.toml").expect("failed to read config.toml");

        let config: DeserializeConfig = toml::from_str(&data).expect("invalid config.toml");

        let mut images = Vec::new();
        let re = Regex::new(
            r"#(?P<priority>\d*)-(?P<name>\S*)\((?P<x>\d*),(?P<y>\d*)\).(?P<extension>\S*)",
        )
        .unwrap();

        for entry in fs::read_dir("./images").expect("failed to read images/") {
            let entry = entry.unwrap();
            let file_name = entry.file_name().into_string().unwrap();

            if let Some(caps) = re.captures(&file_name) {
                let data = read_image(entry.path(), caps["extension"].to_string());
                let len_x = data.len();
                let len_y = data[0].len();

                images.push(Image {
                    name: caps["name"].to_string(),
                    x: caps["x"].parse::<usize>().unwrap(),
                    y: caps["y"].parse::<usize>().unwrap(),
                    priority: caps["priority"].parse::<i32>().unwrap(),
                    len_x,
                    len_y,
                    data,
                });
            }
        }

        images.sort_unstable_by_key(|image| image.priority);

        println!("loaded {} images", images.len());

        Config {
            images,
            board_url: format!("{}/board", config.base_url),
            paint_url: format!("{}/paint", config.base_url),
            tokens: config.tokens,
            board_x: config.board_x,
            board_y: config.board_y,
            fetch_interval: config.fetch_interval,
            paint_interval: config.paint_interval,
            max_concurrent_paint: config.max_concurrent_paint,
        }
    };
}

#[derive(Deserialize, Clone, Debug)]
pub struct PaintResponse {
    pub data: String,
    pub status: i32,
}

#[inline(always)]
pub fn char_to_color(char: u8) -> u8 {
    if char < 58 {
        char - 48
    } else {
        char + 10 - 97
    }
}
