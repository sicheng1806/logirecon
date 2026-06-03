use super::provider::SheetProvider;
use crate::{Error, Result, excel::ExcelReadOptions, pipeline::Schema};
use polars::prelude::DataFrame;

pub trait Parser {
    fn provider(&self) -> &SheetProvider;

    fn provider_mut(&mut self) -> &mut SheetProvider;

    /// 解析从sheet中读取到的数据表
    fn parse_dataframe(&self, dataframe: DataFrame) -> Result<DataFrame>;

    /// 返回解析数据表的方案
    fn schema() -> Schema;

    /// 获取解析后的完整数据表
    fn parse(&self) -> Result<DataFrame> {
        use polars::prelude::*;
        //
        let provider = self.provider();
        let primary = provider.primary();
        let headers = provider.headers();
        let opts = ExcelReadOptions::default()
            .with_headers(headers.values())
            .with_primary(primary);
        let schema = Self::schema();

        let mut dataframe = None;
        for (path, sheet) in provider.sheets() {
            let df = opts
                .clone()
                .with_path(path)
                .with_sheet(sheet)
                .try_into_reader()?
                .finish()?;
            let parsed_df = self.parse_dataframe(df)?.lazy();
            let parsed_df = schema.standardlize(parsed_df)?;

            dataframe = if let Some(old_df) = dataframe {
                Some(concat([old_df, parsed_df.lazy()], UnionArgs::default())?)
            } else {
                Some(parsed_df)
            }
        }
        Ok(dataframe.ok_or(Error::IO("没有表格".into()))?.collect()?)
    }
}
