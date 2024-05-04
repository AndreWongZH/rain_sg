use std::{fs::{self, File}, io::Cursor};
use log::info;
use reqwest;
use std::io::copy;
use chrono::NaiveDateTime;
use std::error;

pub const IMG_DIR: &str = "./img";
pub const GIF_DIR: &str = "./gif";
pub const DIR_NAME : &str = "%Y%m%d";
pub const FILE_NAME: &str = "%H%M";
pub const IMAGE_NAME: &str = "%Y%m%d%H%M";

pub struct ImageMeta {
    pub datetime: NaiveDateTime,
}

impl ImageMeta {
    pub fn build_from_str(url: &str) -> Result<ImageMeta, Box<dyn  error::Error>> {
        let datetime = NaiveDateTime::parse_from_str(url, IMAGE_NAME)?;

        Ok(ImageMeta {
            datetime
        })
    }

    pub fn build_from_datetime(datetime: NaiveDateTime) -> Result<ImageMeta, Box<dyn error::Error>> {
        Ok(ImageMeta {
            datetime
        })
    }

    pub async fn download_img(&self) -> Result<(), Box<dyn error::Error>> {
        if self.exist() {
            return Ok(());
        }

        info!("image name {} does not exist, downloading from api now", self.image_name());
        let resp = reqwest::get(self.url_name()).await?;
        if resp.status().is_success() {
            let file_path = self.image_path();
            // check if date dir exist, if not create
            if let Some(parent_dir) = std::path::Path::new(&file_path).parent() {
                if !parent_dir.exists() {
                    info!("Dir {} does not exist, creating it now", parent_dir.display());
                    fs::create_dir_all(parent_dir)?
                }
            }

            // create file
            let mut img_file = File::create(file_path)?;

            let mut content = Cursor::new(resp.bytes().await?);
            copy(&mut content, &mut img_file)?;
        }

        Ok(())
    }

    pub fn file_name(&self) -> String {
        self.datetime.format(FILE_NAME).to_string()
    }

    pub fn dir_name(&self) -> String {
        self.datetime.format(DIR_NAME).to_string()
    }

    pub fn image_name(&self) -> String {
        self.datetime.format(IMAGE_NAME).to_string()
    }

    pub fn image_path(&self) -> String {
        format!("{}/{}/{}.png", IMG_DIR, self.dir_name(), self.file_name())
    }

    pub fn url_name(&self) -> String {
        format!("http://www.weather.gov.sg/files/rainarea/50km/v2/dpsri_70km_{}0000dBR.dpsri.png", self.image_name())
    }

    pub fn exist(&self) -> bool {
        std::path::Path::new(&self.image_path()).exists()
    }
}