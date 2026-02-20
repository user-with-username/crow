mod dev;
mod release;
mod test;
mod bench;
mod profile_enum;

pub use dev::DevProfile;
pub use release::ReleaseProfile;
pub use test::TestProfile;
pub use bench::BenchProfile;
pub use profile_enum::Profile;

use serde::Deserialize;
use smart_default::SmartDefault;

#[derive(Deserialize, SmartDefault)]
pub struct Profiles {
    #[serde(default)]
    pub dev: DevProfile,
    #[serde(default)]
    pub release: ReleaseProfile,
    #[serde(default)]
    pub test: TestProfile,
    #[serde(default)]
    pub bench: BenchProfile,
}