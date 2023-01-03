use anyhow::Result;
use futures_util::TryFutureExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use lazy_static::lazy_static;
use luogu_paintboard::{char_to_color, Board, PaintResponse, CONFIG};
use parking_lot::Mutex;
use rayon::prelude::*;
use reqwest::Client;
use tokio::time::{sleep, Duration};

#[macro_use]
extern crate log;

lazy_static! {
    static ref CLIENT: Client = Client::new();
    static ref UNFINISHED: Mutex<Vec<(usize, usize, u8)>> = Mutex::new(Vec::new());
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
    let token_amount = CONFIG.tokens.len();

    tokio::spawn(async move {
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

                    let mut unfinished = Vec::new();

                    CONFIG.images.iter().enumerate().for_each(|(id, image)| {
                        let mut tmp = Vec::new();

                        #[allow(clippy::needless_range_loop)]
                        for x in image.x..image.x + image.len_x {
                            for y in image.y..image.y + image.len_y {
                                let board_color = char_to_color(board[x][y]);
                                let image_color = image.data[x - image.x][y - image.y];

                                if board_color != image_color {
                                    tmp.push((x, y, image_color));
                                }
                            }
                        }

                        PBS[id].set_position((image.len_x * image.len_y - tmp.len()) as u64);

                        if unfinished.len() < token_amount * 5 {
                            fastrand::shuffle(&mut tmp);
                            unfinished.append(&mut tmp);
                        }
                    });

                    unfinished.truncate(token_amount * 5);

                    *UNFINISHED.lock() = unfinished;

                    info!("successfully read borad");
                }
                other => error!("failed to read board ({other:?})"),
            }

            sleep(Duration::from_secs(CONFIG.fetch_interval)).await;
        }
    });

    loop {
        let req: Vec<_>;

        {
            let mut unfinished = UNFINISHED.lock();
            let max_len = unfinished.len();

            req = unfinished
                .par_drain(..(token_amount - token_pos).min(max_len))
                .enumerate()
                .map(|(id, (x, y, color))| {
                    [
                        ("x", x.to_string()),
                        ("y", y.to_string()),
                        ("color", color.to_string()),
                        ("uid", CONFIG.tokens[token_pos + id].uid.to_string()),
                        ("token", CONFIG.tokens[token_pos + id].token.to_string()),
                    ]
                })
                .collect();
        }

        token_pos += req.len();

        if token_pos >= token_amount {
            token_pos = 0;
        }

        let mut handles_cnt = 0;

        for params in req {
            let handle = tokio::spawn(async move {
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

            handles_cnt += 1;

            if handles_cnt >= CONFIG.max_concurrent_paint {
                handle.await?;
                handles_cnt = 0;
            }
        }

        sleep(Duration::from_secs(CONFIG.paint_interval)).await;
    }
}
