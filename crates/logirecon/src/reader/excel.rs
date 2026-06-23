//! 从Excel文件中读取表格

use super::{ExcelError, excel_impl};
use crate::DataFrame;
use std::{
    collections::{HashMap, HashSet},
    io::{Read, Seek},
    path::PathBuf,
};

use calamine::{Data, Range, Reader};

/// 从Excel文件中读取表格
///
/// 使用 [calamine] 实现
///
/// # Example
///
/// ```ignore
/// let df = ExcelReader::new(headers)
///     .load_worksheet(&path, sheet)?
///     .primary("序号")
///     .read()?;
///
/// println!("{}", df);
/// ```
pub struct ExcelReader {
    data: Option<Range<Data>>,
    headers: Vec<String>,
    primary: Option<String>,
}

impl ExcelReader {
    pub fn new(headers: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            data: None,
            headers: headers.into_iter().map(|t| t.into()).collect(),
            primary: None,
        }
    }

    pub fn primary(mut self, name: &str) -> Self {
        self.primary = Some(name.to_string());
        self
    }

    pub fn load_worksheet(
        mut self,
        path: impl Into<PathBuf>,
        name: &str,
    ) -> Result<Self, ExcelError> {
        if let Some(_) = self.data.as_ref() {
            Err(ExcelError::DuplicateLoad)
        } else {
            let mut wb = calamine::open_workbook_auto(path.into())?;
            self.data = Some(wb.worksheet_range(name)?);
            Ok(self)
        }
    }

    pub fn load_worksheet_from_rs<RS>(
        mut self,
        rs: RS,
        name: &str,
        extension: &str,
    ) -> Result<Self, ExcelError>
    where
        RS: Read + Seek,
    {
        use calamine::{Ods, Sheets, Xls, Xlsb, Xlsx, open_workbook_from_rs};
        let mut wb = match extension {
            "xls" | "xla" => {
                Sheets::Xls(open_workbook_from_rs::<Xls<RS>, RS>(rs).map_err(calamine::Error::Xls)?)
            }
            "xlsx" | "xlsm" | "xlam" => Sheets::Xlsx(
                open_workbook_from_rs::<Xlsx<RS>, RS>(rs).map_err(calamine::Error::Xlsx)?,
            ),
            "xlsb" => Sheets::Xlsb(
                open_workbook_from_rs::<Xlsb<RS>, RS>(rs).map_err(calamine::Error::Xlsb)?,
            ),
            "ods" => {
                Sheets::Ods(open_workbook_from_rs::<Ods<RS>, RS>(rs).map_err(calamine::Error::Ods)?)
            }
            _ => {
                return Err(ExcelError::Load(calamine::Error::Msg(
                    "Cannot detect file format",
                )));
            }
        };
        self.data = Some(wb.worksheet_range(name)?);
        Ok(self)
    }

    pub fn read(self) -> Result<DataFrame, ExcelError> {
        let Self {
            data,
            headers,
            primary,
        } = self;
        if let Some(data) = data {
            let primary = primary.unwrap_or(headers[0].clone());
            let headers: HashSet<String> = HashSet::from_iter(headers.into_iter());
            let (data, headers) = excel_impl::find_data_scope(data, headers, primary)?;
            let headers: HashMap<u32, String> = headers.into_iter().map(|(k, v)| (v, k)).collect();
            let df = excel_impl::read_by_data_scope(data, headers)?;
            Ok(df)
        } else {
            Err(ExcelError::NotLoad)
        }
    }
}
