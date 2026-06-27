use crate::builder::{flags::flag::ArchiverFlag, kinds::archiver_kind::ArchiverKind};

#[derive(Debug, Clone)]
pub struct ArchiverFlags {
    archiver_kind: ArchiverKind,
    flags: Vec<ArchiverFlag>,
}

impl ArchiverFlags {
    pub fn new(archiver_kind: ArchiverKind) -> Self {
        Self {
            archiver_kind,
            flags: Vec::new(),
        }
    }

    pub fn output_file(&mut self, path: impl Into<String>) -> &mut Self {
        self.flags.push(ArchiverFlag::OutputFile(path.into()));
        self
    }

    pub fn add_object(&mut self, object: impl Into<String>) -> &mut Self {
        self.flags.push(ArchiverFlag::Object(object.into()));
        self
    }

    pub fn add_raw(&mut self, raw: impl Into<String>) -> &mut Self {
        self.flags.push(ArchiverFlag::Raw(raw.into()));
        self
    }

    pub fn build(&self) -> Vec<String> {
        let mut args = Vec::new();
        match self.archiver_kind {
            ArchiverKind::Ar | ArchiverKind::LlvmAr => {
                args.push("rcs".to_string());
                for flag in &self.flags {
                    match flag {
                        ArchiverFlag::OutputFile(path) => args.push(path.clone()),
                        ArchiverFlag::Object(obj) => args.push(obj.clone()),
                        ArchiverFlag::Raw(raw) => args.push(raw.clone()),
                    }
                }
            }
            ArchiverKind::Lib => {
                for flag in &self.flags {
                    match flag {
                        ArchiverFlag::OutputFile(path) => args.push(format!("/OUT:{}", path)),
                        ArchiverFlag::Object(obj) => args.push(obj.clone()),
                        ArchiverFlag::Raw(raw) => args.push(raw.clone()),
                    }
                }
            }
            ArchiverKind::Unknown => {
                for flag in &self.flags {
                    match flag {
                        ArchiverFlag::OutputFile(path) => args.push(path.clone()),
                        ArchiverFlag::Object(obj) => args.push(obj.clone()),
                        ArchiverFlag::Raw(raw) => args.push(raw.clone()),
                    }
                }
            }
        }
        args
    }
}
