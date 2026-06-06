use super::{SheetProvider, Validated};
use crate::{DataFrame, LazyFrame, Result};

/// Parse 特征
///
/// Parse 特征用于实现基于 [SheetProvider] 和 [Validated] 的用于表单数据解析器。
///
/// SheetProvider 用于提供指定表头的数据，而 Validated 特征用于指定 parse 方法
/// 返回的数据验证器，表示解析器提供的数据是需通过特定验证器的数据。
///
/// # Example
/// ```ignore
/// let mut parser = HeadwayParser::default();
/// parser
///     .provider_mut()
///     .add_sheets(PATH_HEADWAY, SHEET_HEADWAY_2026)
///     .update_headers([("报关费", "报关或其他费")]);
/// let df = parser.parse().unwrap().get_valicated().unwrap();
/// println!("{}", df);
/// ```
pub trait Parse<T: Validated> {
    /// 获取提供数据的[SheetProvider]
    fn provider(&self) -> &SheetProvider;

    /// 获取提供数据的[SheetProvider]
    fn provider_mut(&mut self) -> &mut SheetProvider;

    /// 解析从Sheet中读取到的数据表
    fn parse_dataframe(&self, dataframe: DataFrame) -> Result<LazyFrame>;

    /// 获取解析后完整的数据表
    fn parse(&self) -> Result<T> {
        use polars::prelude::*;

        let provider = self.provider();
        let mut dfs: Vec<_> = vec![];
        for df_res in provider.try_get_dataframes() {
            let parsed_df = self.parse_dataframe(df_res?)?;
            dfs.push(parsed_df);
        }
        Ok(T::with_dataframe(
            concat(&dfs, UnionArgs::default())?.collect()?,
        ))
    }
}
