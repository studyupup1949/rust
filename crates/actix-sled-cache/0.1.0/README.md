# Sled Actix Cache
_A caching system built on top of Sled and Actix_

- [Join the discussion on Matrix](https://matrix.to/#/!TIMspMiwmcgOuqnOgt:asonix.dog?via=asonix.dog)

This project is designed to allow easy caching in actix-based systems. Specifically, it was
developed to hold image files and metadata in a media cache for a simple web application.

### Example:
```rust
use actix::{Actor, System};
use actix_sled_cache::bincode::Cache;
use sled_extensions::{Db, ConfigBuilder};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sys = System::new("cache-example");
    let db = Db::start(ConfigBuilder::default().temporary(true).build())?;

    let mut cache_builder = Cache::builder(db, "simple-cache");
    cache_builder
        .as_mut()
        .extend_on_update()
        .extend_on_fetch()
        .expiration_length(chrono::Duration::seconds(1));

    let cache: Cache<usize> = cache_builder
        .frequency(std::time::Duration::from_secs(1))
        .build()?;

    // Clone the tree out of the cache before starting the cache actor
    let tree = cache.tree();

    cache.start();

    // If un-accessed, this record will be deleted after one second
    tree.insert("some-key", 5)?;

    sys.run()?;
    Ok(())
}
```

The cache created here has a frequency of one second, which means every second it will check
for expired records. In many cases, the frequency can be much less often. This cache also has
an expiration_length of one second, which means after one second has passed since the last
interaction with the record, it is eligible for deletion.

### Contributing
Unless otherwise stated, all contributions to this project will be licensed under the CSL with
the exceptions listed in the License section of this file.

### License
This work is licensed under the Cooperative Software License. This is not a Free Software
License, but may be considered a "source-available License." For most hobbyists, self-employed
developers, worker-owned companies, and cooperatives, this software can be used in most
projects so long as this software is distributed under the terms of the CSL. For more
information, see the provided LICENSE file. If none exists, the license can be found online
[here](https://lynnesbian.space/csl/). If you are a free software project and wish to use this
software under the terms of the GNU Affero General Public License, please contact me at
[asonix@asonix.dog](mailto:asonix@asonix.dog) and we can sort that out. If you wish to use this
project under any other license, especially in proprietary software, the answer is likely no.
