extern crate rand;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate structopt;
extern crate url;

use std::fs::File;
use std::num::ParseIntError;

use rand::{thread_rng, Rng};
use reqwest::{Client, Error as ReqwestError};
use structopt::StructOpt;
use url::Url;

const BASE_URL: &str = "https://wall.alphacoders.com/api2.0/get.php";
const IMAGES_PER_PAGE: f32 = 30.0;

#[derive(StructOpt)]
#[structopt(name = "wally")]
struct Opt {
    #[structopt(short = "k", long = "api-key")]
    api_key: String,

    #[structopt(short = "w", long = "width")]
    width: u32,

    #[structopt(short = "h", long = "height")]
    height: u32,
}

#[derive(Debug)]
enum Error {
    WallpaperError(String),
    Http(ReqwestError),
    IntParse(ParseIntError),
}

impl From<ReqwestError> for Error {
    fn from(err: ReqwestError) -> Error {
        Error::Http(err)
    }
}

impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Error {
        Error::IntParse(err)
    }
}

#[derive(Deserialize)]
struct WallpaperCountResponse {
    success: bool,
    error: Option<String>,
    count: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Wallpaper {
    id: String,
    width: String,
    height: String,
    url_image: String,
}

#[derive(Deserialize)]
struct WallpapersResponse {
    success: bool,
    error: Option<String>,
    wallpapers: Option<Vec<Wallpaper>>,
}

fn base_url(api_key: &str, method: &str) -> String {
    format!("{}?auth={}&method={}", BASE_URL, api_key, method)
}

fn get_popular_wallpapers(
    client: &Client,
    api_key: &str,
    width: u32,
    height: u32,
    page: u32,
) -> Result<Vec<Wallpaper>, Error> {
    let url = format!(
        "{}&width={}&height={}&page={}",
        base_url(api_key, "popular"),
        width,
        height,
        page
    );

    let response: WallpapersResponse = client.get(&url).send()?.json()?;
    if response.success {
        Ok(response.wallpapers.unwrap())
    } else {
        Err(Error::WallpaperError(
            response.error.unwrap_or("API Error".to_string()),
        ))
    }
}

fn get_popular_wallpaper_count(
    client: &Client,
    api_key: &str,
    width: u32,
    height: u32,
) -> Result<u32, Error> {
    let url = format!(
        "{}&width={}&height={}",
        base_url(api_key, "popular_count"),
        width,
        height
    );
    let response: WallpaperCountResponse = client.get(&url).send()?.json()?;
    if response.success {
        Ok(response.count.unwrap().parse()?)
    } else {
        Err(Error::WallpaperError(
            response.error.unwrap_or("API Error".to_string()),
        ))
    }
}

fn main() {
    // parse command line args
    let opt = Opt::from_args();

    // first get count of popular wallpapers
    let client = Client::new();
    let count = get_popular_wallpaper_count(&client, &opt.api_key, opt.width, opt.height).unwrap();

    // pick a wallpaper at random
    let mut rng = thread_rng();
    let index = rng.gen_range(0, count + 1);

    // get the page which has the selected wallpaper
    let page = (index as f32 / IMAGES_PER_PAGE).ceil() as u32;
    let wallpapers =
        get_popular_wallpapers(&client, &opt.api_key, opt.width, opt.height, page).unwrap();

    // download the wallpaper
    let entry = &wallpapers[(index % IMAGES_PER_PAGE as u32) as usize];
    let image_url: Url = entry.url_image.parse().unwrap();
    let file_name = image_url.path_segments().unwrap().last().unwrap();
    let file_name = format!("/tmp/_wally-{}", file_name);
    let mut output_file = File::create(&file_name).unwrap();
    client
        .get(&entry.url_image)
        .send()
        .unwrap()
        .copy_to(&mut output_file)
        .unwrap();
    println!("{}", file_name);
}
