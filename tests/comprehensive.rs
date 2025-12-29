use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;

use derive_from_env::{FromEnv, FromEnvError};
use temp_env::with_vars;

// =============================================================================
// Basic struct tests (no nesting)
// =============================================================================

#[derive(FromEnv, Debug, PartialEq)]
struct SimpleStruct {
    name: String,
    count: u32,
}

#[test]
fn test_simple_struct() {
    with_vars(vec![("NAME", Some("hello")), ("COUNT", Some("42"))], || {
        let s = SimpleStruct::from_env().unwrap();
        assert_eq!(
            s,
            SimpleStruct {
                name: "hello".into(),
                count: 42
            }
        );
    })
}

#[derive(FromEnv, Debug, PartialEq)]
#[from_env(prefix = "APP_")]
struct SimpleStructWithPrefix {
    name: String,
    count: u32,
}

#[test]
fn test_simple_struct_with_prefix() {
    with_vars(
        vec![("APP_NAME", Some("world")), ("APP_COUNT", Some("100"))],
        || {
            let s = SimpleStructWithPrefix::from_env().unwrap();
            assert_eq!(
                s,
                SimpleStructWithPrefix {
                    name: "world".into(),
                    count: 100
                }
            );
        },
    )
}

#[test]
fn test_from_env_with_prefix_override() {
    with_vars(
        vec![
            ("CUSTOM_NAME", Some("custom")),
            ("CUSTOM_COUNT", Some("999")),
        ],
        || {
            let s = SimpleStruct::from_env_with_prefix("CUSTOM").unwrap();
            assert_eq!(
                s,
                SimpleStruct {
                    name: "custom".into(),
                    count: 999
                }
            );
        },
    )
}

// =============================================================================
// All primitive types
// =============================================================================

#[derive(FromEnv, Debug, PartialEq)]
struct AllPrimitives {
    val_i8: i8,
    val_i16: i16,
    val_i32: i32,
    val_i64: i64,
    val_i128: i128,
    val_u8: u8,
    val_u16: u16,
    val_u32: u32,
    val_u64: u64,
    val_u128: u128,
    val_f32: f32,
    val_f64: f64,
    val_bool: bool,
    val_char: char,
    val_isize: isize,
    val_usize: usize,
}

#[test]
fn test_all_primitives() {
    with_vars(
        vec![
            ("VAL_I8", Some("-128")),
            ("VAL_I16", Some("-32000")),
            ("VAL_I32", Some("-2000000")),
            ("VAL_I64", Some("-9000000000")),
            ("VAL_I128", Some("-170141183460469231731687303715884105728")),
            ("VAL_U8", Some("255")),
            ("VAL_U16", Some("65000")),
            ("VAL_U32", Some("4000000000")),
            ("VAL_U64", Some("18000000000000000000")),
            ("VAL_U128", Some("340282366920938463463374607431768211455")),
            ("VAL_F32", Some("1.5")),
            ("VAL_F64", Some("2.7")),
            ("VAL_BOOL", Some("true")),
            ("VAL_CHAR", Some("X")),
            ("VAL_ISIZE", Some("-42")),
            ("VAL_USIZE", Some("42")),
        ],
        || {
            let p = AllPrimitives::from_env().unwrap();
            assert_eq!(p.val_i8, -128);
            assert_eq!(p.val_i16, -32000);
            assert_eq!(p.val_i32, -2000000);
            assert_eq!(p.val_i64, -9000000000);
            assert_eq!(p.val_i128, -170141183460469231731687303715884105728);
            assert_eq!(p.val_u8, 255);
            assert_eq!(p.val_u16, 65000);
            assert_eq!(p.val_u32, 4000000000);
            assert_eq!(p.val_u64, 18000000000000000000);
            assert_eq!(p.val_u128, 340282366920938463463374607431768211455);
            assert!((p.val_f32 - 1.5).abs() < 0.001);
            assert!((p.val_f64 - 2.7).abs() < 0.0000001);
            assert!(p.val_bool);
            assert_eq!(p.val_char, 'X');
            assert_eq!(p.val_isize, -42);
            assert_eq!(p.val_usize, 42);
        },
    )
}

// =============================================================================
// Network and path types (FromStr implementations)
// =============================================================================

#[derive(FromEnv, Debug, PartialEq)]
struct NetworkTypes {
    ip_addr: IpAddr,
    ipv4: Ipv4Addr,
    ipv6: Ipv6Addr,
    socket: SocketAddr,
    path: PathBuf,
}

#[test]
fn test_network_and_path_types() {
    with_vars(
        vec![
            ("IP_ADDR", Some("192.168.1.1")),
            ("IPV4", Some("10.0.0.1")),
            ("IPV6", Some("::1")),
            ("SOCKET", Some("127.0.0.1:8080")),
            ("PATH", Some("/usr/local/bin")),
        ],
        || {
            let n = NetworkTypes::from_env().unwrap();
            assert_eq!(n.ip_addr, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
            assert_eq!(n.ipv4, Ipv4Addr::new(10, 0, 0, 1));
            assert_eq!(n.ipv6, Ipv6Addr::LOCALHOST);
            assert_eq!(n.socket, SocketAddr::from_str("127.0.0.1:8080").unwrap());
            assert_eq!(n.path, PathBuf::from("/usr/local/bin"));
        },
    )
}

// =============================================================================
// Option types
// =============================================================================

#[derive(FromEnv, Debug, PartialEq)]
struct WithOptions {
    required: String,
    optional_string: Option<String>,
    optional_num: Option<i32>,
    optional_bool: Option<bool>,
}

#[test]
fn test_option_all_present() {
    with_vars(
        vec![
            ("REQUIRED", Some("must-have")),
            ("OPTIONAL_STRING", Some("bonus")),
            ("OPTIONAL_NUM", Some("123")),
            ("OPTIONAL_BOOL", Some("false")),
        ],
        || {
            let w = WithOptions::from_env().unwrap();
            assert_eq!(w.required, "must-have");
            assert_eq!(w.optional_string, Some("bonus".into()));
            assert_eq!(w.optional_num, Some(123));
            assert_eq!(w.optional_bool, Some(false));
        },
    )
}

#[test]
fn test_option_none_present() {
    with_vars(vec![("REQUIRED", Some("must-have"))], || {
        let w = WithOptions::from_env().unwrap();
        assert_eq!(w.required, "must-have");
        assert_eq!(w.optional_string, None);
        assert_eq!(w.optional_num, None);
        assert_eq!(w.optional_bool, None);
    })
}

#[test]
fn test_option_partial() {
    with_vars(
        vec![
            ("REQUIRED", Some("must-have")),
            ("OPTIONAL_NUM", Some("456")),
        ],
        || {
            let w = WithOptions::from_env().unwrap();
            assert_eq!(w.required, "must-have");
            assert_eq!(w.optional_string, None);
            assert_eq!(w.optional_num, Some(456));
            assert_eq!(w.optional_bool, None);
        },
    )
}

// =============================================================================
// Default values
// =============================================================================

#[derive(FromEnv, Debug, PartialEq)]
struct WithDefaults {
    #[from_env(default = "default_name")]
    name: String,
    #[from_env(default = "8080")]
    port: u16,
    #[from_env(default = "true")]
    enabled: bool,
    no_default: String,
}

#[test]
fn test_defaults_used() {
    with_vars(
        vec![
            ("NO_DEFAULT", Some("required")),
            // Explicitly unset these to ensure defaults are used
            ("NAME", None),
            ("PORT", None),
            ("ENABLED", None),
        ],
        || {
            let w = WithDefaults::from_env().unwrap();
            assert_eq!(w.name, "default_name");
            assert_eq!(w.port, 8080);
            assert!(w.enabled);
            assert_eq!(w.no_default, "required");
        },
    )
}

#[test]
fn test_defaults_overridden() {
    with_vars(
        vec![
            ("NAME", Some("custom_name")),
            ("PORT", Some("9000")),
            ("ENABLED", Some("false")),
            ("NO_DEFAULT", Some("required")),
        ],
        || {
            let w = WithDefaults::from_env().unwrap();
            assert_eq!(w.name, "custom_name");
            assert_eq!(w.port, 9000);
            assert!(!w.enabled);
            assert_eq!(w.no_default, "required");
        },
    )
}

// =============================================================================
// var attribute (absolute env var name)
// =============================================================================

#[derive(FromEnv, Debug, PartialEq)]
#[from_env(prefix = "MYAPP_")]
struct WithVarAttribute {
    normal_field: String,
    #[from_env(var = "ABSOLUTE_VAR")]
    absolute_field: String,
    #[from_env(var = "ANOTHER_ABSOLUTE", default = "fallback")]
    absolute_with_default: String,
}

#[test]
fn test_var_bypasses_prefix() {
    with_vars(
        vec![
            ("MYAPP_NORMAL_FIELD", Some("normal")),
            ("ABSOLUTE_VAR", Some("absolute")),
            ("ANOTHER_ABSOLUTE", Some("another")),
        ],
        || {
            let w = WithVarAttribute::from_env().unwrap();
            assert_eq!(w.normal_field, "normal");
            assert_eq!(w.absolute_field, "absolute");
            assert_eq!(w.absolute_with_default, "another");
        },
    )
}

#[test]
fn test_var_with_default_fallback() {
    with_vars(
        vec![
            ("MYAPP_NORMAL_FIELD", Some("normal")),
            ("ABSOLUTE_VAR", Some("absolute")),
            // ANOTHER_ABSOLUTE not set, should use default
        ],
        || {
            let w = WithVarAttribute::from_env().unwrap();
            assert_eq!(w.absolute_with_default, "fallback");
        },
    )
}

// =============================================================================
// rename attribute
// =============================================================================

#[derive(FromEnv, Debug, PartialEq)]
#[from_env(prefix = "CFG_")]
struct WithRename {
    #[from_env(rename = "custom_name")]
    field_with_long_name: String,
    normal_field: String,
}

#[test]
fn test_rename_respects_prefix() {
    with_vars(
        vec![
            ("CFG_CUSTOM_NAME", Some("renamed")),
            ("CFG_NORMAL_FIELD", Some("normal")),
        ],
        || {
            let w = WithRename::from_env().unwrap();
            assert_eq!(w.field_with_long_name, "renamed");
            assert_eq!(w.normal_field, "normal");
        },
    )
}

// =============================================================================
// Custom FromStr types
// =============================================================================

#[derive(Debug, PartialEq)]
enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl FromStr for LogLevel {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "debug" => Ok(LogLevel::Debug),
            "info" => Ok(LogLevel::Info),
            "warn" | "warning" => Ok(LogLevel::Warn),
            "error" => Ok(LogLevel::Error),
            _ => Err(format!("Unknown log level: {}", s)),
        }
    }
}

#[derive(FromEnv, Debug, PartialEq)]
struct WithCustomType {
    log_level: LogLevel,
    #[from_env(default = "info")]
    default_level: LogLevel,
    optional_level: Option<LogLevel>,
}

#[test]
fn test_custom_from_str() {
    with_vars(
        vec![
            ("LOG_LEVEL", Some("DEBUG")),
            ("OPTIONAL_LEVEL", Some("error")),
        ],
        || {
            let w = WithCustomType::from_env().unwrap();
            assert_eq!(w.log_level, LogLevel::Debug);
            assert_eq!(w.default_level, LogLevel::Info);
            assert_eq!(w.optional_level, Some(LogLevel::Error));
        },
    )
}

#[test]
fn test_custom_from_str_optional_missing() {
    with_vars(vec![("LOG_LEVEL", Some("warn"))], || {
        let w = WithCustomType::from_env().unwrap();
        assert_eq!(w.log_level, LogLevel::Warn);
        assert_eq!(w.default_level, LogLevel::Info);
        assert_eq!(w.optional_level, None);
    })
}

// =============================================================================
// Nested structs - basic
// =============================================================================

#[derive(FromEnv, Debug, PartialEq)]
struct DatabaseConfig {
    host: String,
    #[from_env(default = "5432")]
    port: u16,
}

#[derive(FromEnv, Debug, PartialEq)]
struct AppWithDatabase {
    name: String,
    #[from_env(flatten)]
    database: DatabaseConfig,
}

#[test]
fn test_nested_struct_basic() {
    with_vars(
        vec![
            ("NAME", Some("myapp")),
            ("DATABASE_HOST", Some("localhost")),
            ("DATABASE_PORT", Some("3306")),
        ],
        || {
            let a = AppWithDatabase::from_env().unwrap();
            assert_eq!(a.name, "myapp");
            assert_eq!(a.database.host, "localhost");
            assert_eq!(a.database.port, 3306);
        },
    )
}

#[test]
fn test_nested_struct_with_default() {
    with_vars(
        vec![
            ("NAME", Some("myapp")),
            ("DATABASE_HOST", Some("localhost")),
            // DATABASE_PORT not set, should use default 5432
        ],
        || {
            let a = AppWithDatabase::from_env().unwrap();
            assert_eq!(a.database.port, 5432);
        },
    )
}

// =============================================================================
// Nested structs - with parent prefix
// =============================================================================

#[derive(FromEnv, Debug, PartialEq)]
#[from_env(prefix = "APP_")]
struct AppWithDatabasePrefixed {
    name: String,
    #[from_env(flatten)]
    database: DatabaseConfig,
}

#[test]
fn test_nested_struct_with_parent_prefix() {
    with_vars(
        vec![
            ("APP_NAME", Some("prefixed-app")),
            ("APP_DATABASE_HOST", Some("db.example.com")),
            ("APP_DATABASE_PORT", Some("5433")),
        ],
        || {
            let a = AppWithDatabasePrefixed::from_env().unwrap();
            assert_eq!(a.name, "prefixed-app");
            assert_eq!(a.database.host, "db.example.com");
            assert_eq!(a.database.port, 5433);
        },
    )
}

// =============================================================================
// Nested structs - child has its own prefix
// =============================================================================

#[derive(FromEnv, Debug, PartialEq)]
#[from_env(prefix = "DB_")]
struct DatabaseConfigWithPrefix {
    host: String,
    port: u16,
}

#[derive(FromEnv, Debug, PartialEq)]
struct AppWithPrefixedDatabase {
    name: String,
    #[from_env(flatten)]
    database: DatabaseConfigWithPrefix,
}

#[test]
fn test_nested_struct_child_prefix_combines() {
    // Parent passes "DATABASE" as prefix, child has "DB_" prefix
    // Combined prefix should be "DATABASE_DB"
    with_vars(
        vec![
            ("NAME", Some("app")),
            ("DATABASE_DB_HOST", Some("combined-host")),
            ("DATABASE_DB_PORT", Some("1234")),
        ],
        || {
            let a = AppWithPrefixedDatabase::from_env().unwrap();
            assert_eq!(a.name, "app");
            assert_eq!(a.database.host, "combined-host");
            assert_eq!(a.database.port, 1234);
        },
    )
}

// =============================================================================
// Nested structs - both have prefix
// =============================================================================

#[derive(FromEnv, Debug, PartialEq)]
#[from_env(prefix = "SVC_")]
struct AppWithBothPrefixes {
    name: String,
    #[from_env(flatten)]
    database: DatabaseConfigWithPrefix,
}

#[test]
fn test_nested_struct_both_prefixes_combine() {
    // Parent has "SVC_" prefix, passes "SVC_DATABASE" to child
    // Child has "DB_" prefix, so final is "SVC_DATABASE_DB"
    with_vars(
        vec![
            ("SVC_NAME", Some("service")),
            ("SVC_DATABASE_DB_HOST", Some("deep-host")),
            ("SVC_DATABASE_DB_PORT", Some("9999")),
        ],
        || {
            let a = AppWithBothPrefixes::from_env().unwrap();
            assert_eq!(a.name, "service");
            assert_eq!(a.database.host, "deep-host");
            assert_eq!(a.database.port, 9999);
        },
    )
}

// =============================================================================
// no_prefix attribute
// =============================================================================

#[derive(FromEnv, Debug, PartialEq)]
#[from_env(prefix = "MAIN_")]
struct WithNoPrefix {
    name: String,
    #[from_env(flatten, no_prefix)]
    database: DatabaseConfig,
}

#[test]
fn test_no_prefix_skips_field_name() {
    // no_prefix means don't add "DATABASE" to the prefix
    // So child gets "MAIN" prefix directly
    with_vars(
        vec![
            ("MAIN_NAME", Some("main-app")),
            ("MAIN_HOST", Some("no-prefix-host")),
            ("MAIN_PORT", Some("7777")),
        ],
        || {
            let w = WithNoPrefix::from_env().unwrap();
            assert_eq!(w.name, "main-app");
            assert_eq!(w.database.host, "no-prefix-host");
            assert_eq!(w.database.port, 7777);
        },
    )
}

#[derive(FromEnv, Debug, PartialEq)]
#[from_env(prefix = "OUTER_")]
struct WithNoPrefixAndChildPrefix {
    #[from_env(flatten, no_prefix)]
    database: DatabaseConfigWithPrefix,
}

#[test]
fn test_no_prefix_with_child_prefix() {
    // no_prefix passes "OUTER" to child
    // Child has "DB_" prefix, so final is "OUTER_DB"
    with_vars(
        vec![
            ("OUTER_DB_HOST", Some("outer-db-host")),
            ("OUTER_DB_PORT", Some("5555")),
        ],
        || {
            let w = WithNoPrefixAndChildPrefix::from_env().unwrap();
            assert_eq!(w.database.host, "outer-db-host");
            assert_eq!(w.database.port, 5555);
        },
    )
}

// =============================================================================
// rename on nested structs
// =============================================================================

#[derive(FromEnv, Debug, PartialEq)]
#[from_env(prefix = "APP_")]
struct WithRenamedNested {
    #[from_env(flatten, rename = "db")]
    database: DatabaseConfig,
}

#[test]
fn test_rename_on_nested_struct() {
    with_vars(
        vec![
            ("APP_DB_HOST", Some("renamed-host")),
            ("APP_DB_PORT", Some("3333")),
        ],
        || {
            let w = WithRenamedNested::from_env().unwrap();
            assert_eq!(w.database.host, "renamed-host");
            assert_eq!(w.database.port, 3333);
        },
    )
}

// =============================================================================
// Deep nesting (3 levels)
// =============================================================================

#[derive(FromEnv, Debug, PartialEq)]
struct ConnectionPool {
    #[from_env(default = "10")]
    max_connections: u32,
    #[from_env(default = "30")]
    timeout_seconds: u32,
}

#[derive(FromEnv, Debug, PartialEq)]
struct DeepDatabase {
    host: String,
    #[from_env(flatten)]
    pool: ConnectionPool,
}

#[derive(FromEnv, Debug, PartialEq)]
#[from_env(prefix = "APP_")]
struct DeepApp {
    name: String,
    #[from_env(flatten)]
    database: DeepDatabase,
}

#[test]
fn test_three_level_nesting() {
    with_vars(
        vec![
            ("APP_NAME", Some("deep-app")),
            ("APP_DATABASE_HOST", Some("deep-host")),
            ("APP_DATABASE_POOL_MAX_CONNECTIONS", Some("50")),
            ("APP_DATABASE_POOL_TIMEOUT_SECONDS", Some("60")),
        ],
        || {
            let a = DeepApp::from_env().unwrap();
            assert_eq!(a.name, "deep-app");
            assert_eq!(a.database.host, "deep-host");
            assert_eq!(a.database.pool.max_connections, 50);
            assert_eq!(a.database.pool.timeout_seconds, 60);
        },
    )
}

#[test]
fn test_three_level_nesting_with_defaults() {
    with_vars(
        vec![
            ("APP_NAME", Some("deep-app")),
            ("APP_DATABASE_HOST", Some("deep-host")),
            // pool settings not set, should use defaults
        ],
        || {
            let a = DeepApp::from_env().unwrap();
            assert_eq!(a.database.pool.max_connections, 10);
            assert_eq!(a.database.pool.timeout_seconds, 30);
        },
    )
}

// =============================================================================
// Multiple nested structs
// =============================================================================

#[derive(FromEnv, Debug, PartialEq)]
struct CacheConfig {
    host: String,
    #[from_env(default = "6379")]
    port: u16,
}

#[derive(FromEnv, Debug, PartialEq)]
#[from_env(prefix = "SVC_")]
struct ServiceWithMultipleNested {
    #[from_env(flatten)]
    database: DatabaseConfig,
    #[from_env(flatten)]
    cache: CacheConfig,
}

#[test]
fn test_multiple_nested_structs() {
    with_vars(
        vec![
            ("SVC_DATABASE_HOST", Some("db-host")),
            ("SVC_DATABASE_PORT", Some("5432")),
            ("SVC_CACHE_HOST", Some("cache-host")),
            ("SVC_CACHE_PORT", Some("6380")),
        ],
        || {
            let s = ServiceWithMultipleNested::from_env().unwrap();
            assert_eq!(s.database.host, "db-host");
            assert_eq!(s.database.port, 5432);
            assert_eq!(s.cache.host, "cache-host");
            assert_eq!(s.cache.port, 6380);
        },
    )
}

// =============================================================================
// Error cases
// =============================================================================

#[derive(FromEnv, Debug, PartialEq)]
struct RequiredFields {
    required_string: String,
    required_num: i32,
}

#[test]
fn test_error_missing_required() {
    with_vars(vec![("REQUIRED_STRING", Some("present"))], || {
        let result = RequiredFields::from_env();
        assert!(result.is_err());
        match result.unwrap_err() {
            FromEnvError::MissingEnvVar { var_name } => {
                assert_eq!(var_name, "REQUIRED_NUM");
            }
            _ => panic!("Expected MissingEnvVar error"),
        }
    })
}

#[test]
fn test_error_parsing_failure() {
    with_vars(
        vec![
            ("REQUIRED_STRING", Some("present")),
            ("REQUIRED_NUM", Some("not_a_number")),
        ],
        || {
            let result = RequiredFields::from_env();
            assert!(result.is_err());
            match result.unwrap_err() {
                FromEnvError::ParsingFailure {
                    var_name,
                    expected_type,
                    ..
                } => {
                    assert_eq!(var_name, "REQUIRED_NUM");
                    assert_eq!(expected_type, "i32");
                }
                _ => panic!("Expected ParsingFailure error"),
            }
        },
    )
}

#[test]
fn test_error_in_nested_struct() {
    with_vars(
        vec![("NAME", Some("app"))],
        // DATABASE_HOST is missing
        || {
            let result = AppWithDatabase::from_env();
            assert!(result.is_err());
            match result.unwrap_err() {
                FromEnvError::MissingEnvVar { var_name } => {
                    assert_eq!(var_name, "DATABASE_HOST");
                }
                _ => panic!("Expected MissingEnvVar error"),
            }
        },
    )
}

#[test]
fn test_error_parsing_in_option() {
    with_vars(
        vec![
            ("REQUIRED", Some("ok")),
            ("OPTIONAL_NUM", Some("not_a_number")),
        ],
        || {
            let result = WithOptions::from_env();
            assert!(result.is_err());
            match result.unwrap_err() {
                FromEnvError::ParsingFailure { var_name, .. } => {
                    assert_eq!(var_name, "OPTIONAL_NUM");
                }
                _ => panic!("Expected ParsingFailure error"),
            }
        },
    )
}

// =============================================================================
// Edge cases
// =============================================================================

#[derive(FromEnv, Debug, PartialEq)]
struct EmptyStringTest {
    value: String,
    optional: Option<String>,
}

#[test]
fn test_empty_string_value() {
    with_vars(vec![("VALUE", Some("")), ("OPTIONAL", Some(""))], || {
        let e = EmptyStringTest::from_env().unwrap();
        assert_eq!(e.value, "");
        assert_eq!(e.optional, Some("".into()));
    })
}

#[derive(FromEnv, Debug, PartialEq)]
struct BoolVariants {
    val_true: bool,
    val_false: bool,
}

#[test]
fn test_bool_parsing() {
    with_vars(
        vec![("VAL_TRUE", Some("true")), ("VAL_FALSE", Some("false"))],
        || {
            let b = BoolVariants::from_env().unwrap();
            assert!(b.val_true);
            assert!(!b.val_false);
        },
    )
}

// =============================================================================
// Combining multiple attributes
// =============================================================================

#[derive(FromEnv, Debug, PartialEq)]
#[from_env(prefix = "COMBO_")]
struct CombinedAttributes {
    #[from_env(rename = "custom", default = "default_val")]
    field_one: String,
    #[from_env(var = "ABSOLUTE")]
    field_two: String,
    optional_field: Option<i32>,
    #[from_env(flatten)]
    nested: DatabaseConfig,
    #[from_env(flatten, no_prefix)]
    flat_nested: DatabaseConfig,
}

#[test]
fn test_combined_attributes() {
    with_vars(
        vec![
            ("COMBO_CUSTOM", Some("custom_value")),
            ("ABSOLUTE", Some("absolute_value")),
            ("COMBO_OPTIONAL_FIELD", Some("42")),
            ("COMBO_NESTED_HOST", Some("nested-host")),
            ("COMBO_NESTED_PORT", Some("1111")),
            ("COMBO_HOST", Some("flat-host")),
            ("COMBO_PORT", Some("2222")),
        ],
        || {
            let c = CombinedAttributes::from_env().unwrap();
            assert_eq!(c.field_one, "custom_value");
            assert_eq!(c.field_two, "absolute_value");
            assert_eq!(c.optional_field, Some(42));
            assert_eq!(c.nested.host, "nested-host");
            assert_eq!(c.nested.port, 1111);
            assert_eq!(c.flat_nested.host, "flat-host");
            assert_eq!(c.flat_nested.port, 2222);
        },
    )
}

#[test]
fn test_combined_attributes_with_defaults() {
    with_vars(
        vec![
            // COMBO_CUSTOM not set, should use default
            ("ABSOLUTE", Some("absolute_value")),
            // COMBO_OPTIONAL_FIELD not set
            ("COMBO_NESTED_HOST", Some("nested-host")),
            // COMBO_NESTED_PORT not set, should use default
            ("COMBO_HOST", Some("flat-host")),
            // COMBO_PORT not set, should use default
        ],
        || {
            let c = CombinedAttributes::from_env().unwrap();
            assert_eq!(c.field_one, "default_val");
            assert_eq!(c.optional_field, None);
            assert_eq!(c.nested.port, 5432);
            assert_eq!(c.flat_nested.port, 5432);
        },
    )
}
