# derive_from_env
Extract type safe structured data from environment variables with procedural derive macros

## Usage
```toml
// Cargo.toml
...
[dependencies]
derive_from_env = "0.1.0"
```
```rust	
use std::net::{IpAddr, Ipv4Addr};
use derive_from_env::FromEnv;

#[derive(Debug, PartialEq, FromEnv)]
struct ServiceConfig {
    api_key: String,
    #[from_env(var = "EXT_SERVICE_URL")]
    base_url: String,
}

#[derive(Debug, PartialEq, FromEnv)]
struct AppConfig {
    #[from_env(default = "8080")]
    port: u16,
    #[from_env(default = "0.0.0.0")]
    addr: IpAddr,
    external_service: ServiceConfig,
}

fn main() {
   std::env::set_var("EXTERNAL_SERVICE_API_KEY", "api-key");
   std::env::set_var("EXT_SERVICE_URL", "http://external.service/api");
   let app_config = AppConfig::from_env().unwrap();
   assert_eq!(app_config, AppConfig {
       port: 8080,
       addr: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
       external_service: ServiceConfig {
           api_key: "api-key".into(),
           base_url: "http://external.service/api".into() 
       }
  });
}
```


