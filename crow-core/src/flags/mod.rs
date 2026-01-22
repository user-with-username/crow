use crate::compiler_kind::CompilerKind;
use crate::config::{BuildConfig, Profile};

mod compile;
mod link;
mod helpers;
mod translate;

pub struct Flags<'a> {
    profile: &'a Profile,
    config: &'a BuildConfig,
    compiler: CompilerKind,
}

impl<'a> Flags<'a> {
    pub fn new(
        profile: &'a Profile,
        config: &'a BuildConfig,
        compiler: CompilerKind,
    ) -> Self {
        Self { profile, config, compiler }
    }
}
