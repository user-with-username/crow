mod bench;
mod dev;
mod profile_enum;
mod release;
mod test;

pub use bench::BenchProfile;
pub use dev::DevProfile;
pub use profile_enum::Profile;
pub use release::ReleaseProfile;
pub use test::TestProfile;

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
