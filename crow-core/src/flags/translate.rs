use std::collections::HashMap;
use once_cell::sync::Lazy;

static GCC_TO_MSVC: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    HashMap::from([
        ("-g", "/Zi"),
        ("-O0", "/Od"),
        ("-O1", "/O1"),
        ("-O2", "/O2"),
        ("-O3", "/O2"),
        ("-Os", "/O1"),
        ("-Oz", "/O1"),
        ("-Wall", "/W3"),
        ("-Wextra", "/W4"),
        ("-Werror", "/WX"),
        ("-flto", "/GL"),
    ])
});

pub fn translate_gcc_to_msvc(flag: &str) -> String {
    if let Some(mapped) = GCC_TO_MSVC.get(flag) {
        return mapped.to_string();
    }

    if let Some(v) = flag.strip_prefix("-D") {
        return format!("/D{}", v);
    }
    if let Some(v) = flag.strip_prefix("-I") {
        return format!("/I{}", v);
    }
    if let Some(v) = flag.strip_prefix("-L") {
        return format!("/LIBPATH:{}", v);
    }
    if let Some(v) = flag.strip_prefix("-l") {
        return if v.ends_with(".lib") {
            v.to_string()
        } else {
            format!("{}.lib", v)
        };
    }

    flag.to_string()
}
