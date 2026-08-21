use std::io::Write;

use adsbx_screenshot::{AdsbxBrowser, AdsbxBrowserOptions};
use anyhow::Result;
// use std::convert::TryFrom;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "adsbx-screenshot")]
struct Opt {
    #[structopt(long = "icaos")]
    icaos: Vec<String>,
    #[structopt(long = "regs")]
    regs: Vec<String>,
    #[structopt(long = "width", default_value = "800")]
    width: u32,
    #[structopt(long = "height", default_value = "600")]
    height: u32,
    #[structopt(long = "show-ads")]
    show_ads: bool,
    #[structopt(long = "show-infoblock")]
    show_infoblock: bool,
    #[structopt(long = "zoom", default_value = "10.0")]
    zoom: f32,
    #[structopt(long = "no-isolation")]
    no_isolation: bool,
    #[structopt(long = "url")]
    url: Option<String>,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let start = std::time::Instant::now();
    let mut browser = AdsbxBrowser::new((opt.width, opt.height))?;
    let options = match opt.url {
        Some(url) => AdsbxBrowserOptions::try_from(url.as_str())?,
        None => AdsbxBrowserOptions {
            icaos: opt.icaos,
            regs: opt.regs,
            delete_ads: !opt.show_ads,
            hide_infoblock: !opt.show_infoblock,
            zoom: opt.zoom,
            no_isolation: opt.no_isolation,
            ..Default::default()
        },
    };
    eprintln!("Screenshotting {}", options.to_url());
    let screenshot = browser.screenshot(&options)?;
    let end = std::time::Instant::now();
    eprintln!("It took {:?} to take the screenshot", end - start);
    std::io::stdout().write_all(&screenshot.data)?;
    Ok(())
}
