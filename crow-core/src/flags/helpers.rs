use super::Flags;
use super::translate::translate_gcc_to_msvc;

impl Flags<'_> {
    pub(crate) fn push_profile_flags(&self, out: &mut Vec<String>) {
        for f in self.profile.flags() {
            out.push(self.translate(&f));
        }
    }

    pub(crate) fn push_user_flags(&self, out: &mut Vec<String>) {
        for f in &self.config.flags {
            out.push(self.translate(f.as_str()));
        }
    }

    fn translate(&self, flag: &str) -> String {
        if self.compiler.is_msvc() {
            translate_gcc_to_msvc(flag)
        } else {
            flag.to_string()
        }
    }
}
