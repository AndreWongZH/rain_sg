use std::{fs, fs::File, io::BufWriter};
use chrono::{Duration, Local, Timelike};
use log::{info, warn};
use tokio::time::Instant;
use image::{Frame, Delay, io::Reader, codecs::gif::GifEncoder};

use crate::image_meta;

#[derive(Clone)]
pub struct Engine {
    base: image::DynamicImage,
}

// use a type alias here for clearer code
pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

impl Engine {
    pub fn init() -> Result<Engine> {
        // create image and gif directory
        fs::create_dir_all(image_meta::IMG_DIR)?;
        fs::create_dir_all(image_meta::GIF_DIR)?;

        let base = Reader::open("base.png")?.decode()?;

        Ok(Engine {
            base,
        })
    }

    /// this functions returns the file_name of the gif
    /// this gif is generated from the image of the current time rounded off to nearest 5 mins and the last 25 images before that
    pub async fn generate_current_weather_condition(&self) -> Result<String> {
        let start_time = Instant::now();
        info!("Preparing to generate current weather condition");
        // get current time rounded down to nearest 5 mins
        let mut current_time = Local::now();
        let remainder = current_time.minute() as i64 % 5;
        current_time = current_time - Duration::minutes(remainder);
        info!("Current Time is: {}", current_time);

        // get the last 25 images
        let mut image_infos = Vec::new();
        for _ in 0..25 {
            let image_name = current_time.format(image_meta::IMAGE_NAME).to_string();
            let image_info = match image_meta::ImageMeta::build_from_str(&image_name) {
                Ok(image_info) => image_info,
                Err(err) => {
                    warn!("Failed to build image meta from string format, err: {}", err);
                    continue
                }
            };
            if let Err(err) = image_info.download_img().await {
                warn!("Unable to download image, err: {}", err);
                continue
            }
            image_infos.push(image_info);
            current_time = current_time - Duration::minutes(5);
        }

        if image_infos.len() < 4 {
            return Err("Not enough image metas".into());
        }

        // stitch them together
        match self.create_gif(image_infos).await {
            Ok(gif_name) => {
                info!("GIF successfully created");
                let duration = start_time.elapsed();
                info!("Time taken for gif to be created: {:.2?}", duration);
                Ok(gif_name)
            },
            Err(err) => {
                warn!("Unable to create gif, err: {}", err);
                Err(err)
            }
        }
    }


    async fn create_gif(&self, image_infos: Vec<image_meta::ImageMeta>) -> Result<String>{
        let gif_name = self.gif_name(&image_infos);
        let gif_path = self.gif_path(&gif_name);
        let mut frame_list: Vec<image::Frame> = Vec::new();

        for image_info in image_infos {
            let image_px = image::io::Reader::open(image_info.image_path())?.decode()?;
            let mut image_px = image_px.resize(self.base.width(), self.base.height(), image::imageops::Nearest).to_rgba8();

            let alpha_channel = 125;
            for y in 0..image_px.height() {
                for x in 0..image_px.width() {
                    let pixel = image_px.get_pixel_mut(x, y);
                    if pixel[0] == 0 && pixel[1] == 0 && pixel[2] == 0 {
                        continue
                    }
                    *pixel = image::Rgba([pixel[0], pixel[1], pixel[2], alpha_channel]);
                }
            }

            let mut layered = self.base.clone().to_rgba8();
            image::imageops::overlay(&mut layered , &image_px, 0, 0);

            let frame = Frame::from_parts(layered, 0, 0, Delay::from_numer_denom_ms(10, 1));
            frame_list.push(frame)
        }

        let file = File::create(&gif_path)?;
        let mut writer = BufWriter::new(file);
        let mut encoder = GifEncoder::new(&mut writer);

        frame_list.reverse();
        encoder.encode_frames(frame_list)?;

        Ok(gif_path)
    }

    fn gif_name(&self, image_infos: &Vec<image_meta::ImageMeta>) -> String {
        let start = image_infos.first().unwrap();
        let end = image_infos.last().unwrap();

        format!("{}_{}.gif", start.image_name(), end.image_name())
    }

    fn gif_path(&self, gif_name: &str) -> String {
        format!("{}/{}", image_meta::GIF_DIR, gif_name)
    }
}
