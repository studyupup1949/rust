# adoptium_api

Small wrapper for a [Adoptium REST API](https://api.adoptium.net/q/swagger-ui).

> [!NOTE]
> All documentation comments are written by a LLM (ChatGPT-3.5/3.0).

## Examples

all examples can be found in a [examples](./examples) directory and runned via `cargo r --example <example_name>`.

### How endpoints are built

```rust
use adoptium_api::v3::prelude::*;
//                ^
//                |
// REST API version which we're using

// Endpoint consists of a 2 things: server and actual endpoint:
let endpoint = Adoptium::production(types::OperationgSystems::new());
//             +--------------------+------------------------------+
//             | server we're using |     actual API endpoint      |
```

You can easily translate any endpoint into a code using explanation below:

```
      Adoptium::production
              |
              |  we've imported "v3" module, so pathes are prefixed with `v3`
              |           |
              |           |  types::OperatingSystems
              |           |    |            |
              ▼           ▼    ▼            ▼
https://api.adoptium.net/v3/types/operating_systems
```

### Basic

Shows how to make simple API call without path and query params.

Run command: `cargo r --example fetch_all_supported_operating_systems`

File: [fetch_all_supported_operating_systems](./examples/fetch_all_supported_operating_systems.rs)

```rust
// Build your request
let endpoint = Adoptium::production(types::OperationgSystems::new());

// Now you can get it's URL
let url = endpoint.try_as_url()?;
println!("URL: {url}");

// Or get a parsed response body if endpoint implements [`GetRequest`] trait.
let response = endpoint.get().await?;
println!("Parsed response: {response:#?}");
```

### JRE downloader

Shows how to download a JRE.

Run command: `cargo r --example download_jre`

File: [download_jre.rs](./examples/download_jre.rs)

Warning: this example will write files into `./jre_download_output` directory.

```rust
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
```

## TODO

1. Handle redirects.
2. More robust errors.

## License

[MIT](./LICENSE) license: feel free to steal, sell, use as a drug, modify, etc.
