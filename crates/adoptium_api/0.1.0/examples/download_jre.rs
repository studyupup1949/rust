use adoptium_api::v3::prelude::*;
use async_compression::tokio::bufread::GzipDecoder;
use futures_util::TryStreamExt;
use tokio_util::io::StreamReader;

// We will download a JRE 8 binary for a linux x64 system.

const DESIRED_VERSION: u8 = 8;
const ARCHITECTURE: Architecture = Architecture::X64;
const OPERATING_SYSTEM: OperatingSystem = OperatingSystem::Linux;
const IMAGE_TYPE: ImageType = ImageType::Jre;
const OUTPUT_PATH: &str = "./jre_download_output";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 1. Build the API endpoint for the desired JVM version:
    let latest_version_endpoint = assets::Latest::new(DESIRED_VERSION, JvmImpl::Hotspot)
        .architecture(ARCHITECTURE)
        .os(OPERATING_SYSTEM)
        .image_type(IMAGE_TYPE);

    // 2. Request the latest version info from Adoptium:
    let latest_version_response = Adoptium::production(latest_version_endpoint).get().await?;

    // 3. Extract the package download URL from the response:
    let version_info = latest_version_response.0.first().expect("No version information found");
    let binary = version_info.binary.as_ref().expect("Missing binary information — cannot download");
    let package = binary.package.as_ref().expect("Missing package information — cannot download");
    let package_url = package.link.as_ref();

    // 4. Get decompressed stream of the package archive:
    let package_data = reqwest::get(package_url).await?;
    let package_data_stream = package_data.bytes_stream().map_err(std::io::Error::other);
    let package_data_reader = StreamReader::new(package_data_stream);
    let package_data_decoder = GzipDecoder::new(package_data_reader);

    // 5. Write all package files from the stream:
    tokio_tar::Archive::new(package_data_decoder).unpack(OUTPUT_PATH).await?;

    Ok(())
}
