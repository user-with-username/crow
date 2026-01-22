use super::Flags;

impl Flags<'_> {
    pub fn compile_flags(&self) -> Vec<String> {
        let mut out = Vec::new();

        self.push_profile_flags(&mut out);
        self.push_user_flags(&mut out);
        
        if self.compiler.is_msvc() {
            out.push("/c".into());
        } else {
            out.push("-c".into());
        }

        out
    }

    pub fn include_dir_flags(&self) -> Vec<String> {
        let inc = if self.compiler.is_msvc() { "/I" } else { "-I" };

        self.config
            .include_dirs
            .iter()
            .map(|d| format!("{}{}", inc, d.display()))
            .collect()
    }
}
