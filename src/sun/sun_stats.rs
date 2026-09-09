use crate::sun::sun_info::SunInfo;

pub enum SunStats {
    Calculated(SunInfo),
    Override(i32),
}

impl SunStats {
    pub fn has_override(&self) -> bool {
        matches!(self, SunStats::Override(_))
    }
}