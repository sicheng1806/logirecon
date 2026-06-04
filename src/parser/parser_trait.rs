use super::provider::SheetProvider;
use crate::{Error, Result, Standardlize, excel::ExcelReadOptions};
use polars::prelude::DataFrame;

pub trait Parser: Standardlize {
    fn provider(&self) -> &SheetProvider;

    fn provider_mut(&mut self) -> &mut SheetProvider;

    /// 解析从sheet中读取到的数据表
    fn parse_dataframe(&self, dataframe: DataFrame) -> Result<DataFrame>;

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

        let mut dataframe = None;
        for (path, sheet) in provider.sheets() {
            let df = opts
                .clone()
                .with_path(path)
                .with_sheet(sheet)
                .try_into_reader()?
                .finish()?;
            let parsed_df = self.parse_dataframe(df)?.lazy();
            let parsed_df = self.standardlize(parsed_df)?;

            dataframe = if let Some(old_df) = dataframe {
                Some(concat([old_df, parsed_df.lazy()], UnionArgs::default())?)
            } else {
                Some(parsed_df)
            }
        }
        Ok(dataframe.ok_or(Error::IO("没有表格".into()))?.collect()?)
    }
}
