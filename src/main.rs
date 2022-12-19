use anyhow::Result;
use futures_util::TryFutureExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use luogu_paintboard::{char_to_color, Board, PaintResponse, CONFIG};
use rayon::prelude::*;
use reqwest::Client;
use std::collections::VecDeque;
use tokio::{
    sync::Mutex,
    time::{sleep, Duration},
};

#[macro_use]
extern crate log;

lazy_static! {
    static ref CLIENT: Client = Client::new();
    static ref UNFINISHED: Mutex<VecDeque<(usize, usize, u8)>> = Mutex::new(VecDeque::new());
    static ref PBS: Vec<ProgressBar> = {
        let mp = MultiProgress::new();
        let pb_style = ProgressStyle::with_template("{bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
            .unwrap()
            .progress_chars("##-");

        let mut pbs = Vec::new();

        for image in &CONFIG.images {
            let pb = mp.add(ProgressBar::new((image.len_x * image.len_y) as u64));
            pb.set_style(pb_style.clone());
            pb.set_message(&image.name);
            pbs.push(pb);
        }

        pbs
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Warn)
        .init();

    let mut token_pos = 0;

    tokio::spawn(async {
        loop {
            let board_data = CLIENT
                .get(&CONFIG.board_url)
                .send()
                .and_then(|data| data.text())
                .await;

            match board_data {
                Ok(data) if data.len() > CONFIG.board_x * CONFIG.board_y => {
                    let board: Board = data
                        .par_split('\n')
                        .map(|row| row.as_bytes().to_vec())
                        .collect();

                    let mut unfinished = VecDeque::new();

                    CONFIG.images.iter().enumerate().for_each(|(id, image)| {
                        let mut cnt = 0;

                        #[allow(clippy::needless_range_loop)]
                        for x in image.x..image.x + image.len_x {
                            for y in image.y..image.y + image.len_y {
                                let board_color = char_to_color(board[x][y]);
                                let image_color = image.data[x - image.x][y - image.y];

                                if board_color != image_color {
                                    unfinished.push_back((x, y, image_color));
                                    cnt += 1;
                                }
                            }
                        }

                        PBS[id].set_position((image.len_x * image.len_y - cnt) as u64);
                    });

                    *UNFINISHED.lock().await = unfinished;

                    info!("successfully read borad");
                }
                other => error!("failed to read board ({other:?})"),
            }

            sleep(Duration::from_secs(CONFIG.fetch_interval)).await;
        }
    });

    loop {
        let mut unfinished = UNFINISHED.lock().await;

        while !unfinished.is_empty() {
            let (x, y, color) = unfinished.pop_front().unwrap();

            let params = [
                ("x", x.to_string()),
                ("y", y.to_string()),
                ("color", color.to_string()),
                ("uid", CONFIG.tokens[token_pos].uid.to_string()),
                ("token", CONFIG.tokens[token_pos].token.to_string()),
            ];

            tokio::spawn(async move {
                if let Ok(res) = CLIENT.post(&CONFIG.paint_url).form(&params).send().await {
                    if let Ok(s) = res.text().await {
                        match serde_json::from_str::<PaintResponse>(&s) {
                            Ok(data) => match data.status {
                                200 => info!("{:?}", data.data),
                                _ => warn!("{:?}", data.data),
                            },
                            Err(_) => error!("{}", s),
                        }
                    }
                }
            });

            token_pos += 1;
            if token_pos >= CONFIG.tokens.len() {
                token_pos = 0;
                break;
            }
        }

        drop(unfinished);

        sleep(Duration::from_secs(CONFIG.paint_interval)).await;
    }
}
