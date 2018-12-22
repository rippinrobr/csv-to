use failure::Error;
use serde_json;
use std::fs;

use crate::cache::{Cache, CacheService};

pub struct JsonCache{
    cache_dir: String,
}

impl JsonCache {
    pub fn new(cache_dir: String) -> Self {
        Self {
            cache_dir,
        }
    }
}

impl CacheService for JsonCache {
    fn read(self, name: String) -> Result<Cache, Error> {
        Err(failure::err_msg("JsonCache::read not implemented"))
    }

    fn write(self, cache: Cache) -> Result<(), failure::Error> {

        // 0. Check to see if ./cache exists
        match fs::metadata(self.cache_dir.clone()) {
            Err(e) => {
                match fs::create_dir(&self.cache_dir) {
                    Err(e) => return Err(failure::err_msg(format!("{}", e))),
                    Ok(_) => println!("created cache directory {}", &self.cache_dir),
                }
            },
            Ok(meta) => {
                if meta.is_file() {
                    return Err(failure::err_msg("Cache must be a directory, not a file."));
                }
            }
        }

        // 2. construct the path to the cache file
        let cache_file_path = format!("{}/{}.json", &self.cache_dir, cache.name);

        // 3. Write the JSON out to the file
        match std::fs::write(cache_file_path.clone(), serde_json::to_string(&cache).unwrap()) {
            Ok(_) => {
                println!("Cache file '{}' was saved", cache_file_path);
                Ok(())
            },
            Err(e) => Err(failure::err_msg(format!("error creating cache file '{}'\n{}", &cache_file_path, e))),
        }
    }
}