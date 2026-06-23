use crate::DataFrame;
use crate::validate::IntoValidated;

/// `Parse` Trait 允许将 DataFrame 按照规则清洗为实现了 [IntoValidated] Trait的对象.
pub trait Parse {
    type Output: IntoValidated;
    type Config;
    type Error;

    fn parse(data: DataFrame, config: Self::Config) -> Result<Self::Output, Self::Error>;
}
