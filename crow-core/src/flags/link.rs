use super::Flags;

impl Flags<'_> {
    pub fn link_flags(&self) -> Vec<String> {
        let mut out = Vec::new();

        self.push_profile_flags(&mut out);
        self.push_library_flags(&mut out);

        out
    }

    fn push_library_flags(&self, out: &mut Vec<String>) {
        if self.compiler.is_msvc() {
            for dir in &self.config.lib_dirs {
                out.push(format!("/LIBPATH:{}", dir.display()));
            }
            for lib in &self.config.libs {
                out.push(if lib.ends_with(".lib") {
                    lib.clone()
                } else {
                    format!("{}.lib", lib)
                });
            }
        } else {
            for dir in &self.config.lib_dirs {
                out.push("-L".into());
                out.push(dir.display().to_string());
            }
            for lib in &self.config.libs {
                out.push("-l".into());
                out.push(lib.clone());
            }
        }
    }
}
