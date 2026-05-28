pub mod utils;
pub mod error;

pub use error::Result;

#[cfg(test)]
mod tests {
    mod calamine {
        use calamine::{Reader, Xlsx, open_workbook};
        use std::path::Path;

        #[test]
        fn test_calamine() {
            let path = Path::new("data/test/iris.xlsx");
            assert!(path.exists());
            let mut workbook: Xlsx<_> = open_workbook(path).expect("无法读取xlsx文件");
            assert_eq!(workbook.sheet_names(), vec!["Sheet1".to_string(),]);
            // dbg!(workbook.has_1904_epoch());
            // dbg!(workbook.metadata());
            let sheet = workbook.worksheet_range("Sheet1").unwrap();
            assert_eq!(sheet.range((0, 0), (0, 4)).width(), 5);
            assert_eq!(
                sheet.headers().unwrap(),
                vec![
                    "No.",
                    "Sepal.Length",
                    "Sepal.Width",
                    "Petal.Lenght",
                    "Patal.Width",
                    "Species",
                ]
            );
            // dbg!(workbook.defined_names());
            for row in sheet.rows() {
                println!(
                    "{}\t{}\t{}\t{}\t{}\t{}",
                    row[0], row[1], row[2], row[3], row[4], row[5],
                )
            }
        }
    }

    mod polars {
        use chrono::prelude::*;
        use polars::prelude::*;
        use std::fs::File;

        #[test]
        fn test_dataframe() {
            let mut df: DataFrame = df!(
                "name" => ["Alice Archer", "Ben Brown", "Chloe Cooper", "Daniel Donovan"],
                "birthdate" => [
                    NaiveDate::from_ymd_opt(1997, 1, 10).unwrap(),
                    NaiveDate::from_ymd_opt(1985, 2, 15).unwrap(),
                    NaiveDate::from_ymd_opt(1983, 3, 22).unwrap(),
                    NaiveDate::from_ymd_opt(1981, 4, 30).unwrap(),
                ],
                "weight" => [57.9, 72.5, 53.6, 83.1],  // (kg)
                "height" => [1.56, 1.77, 1.65, 1.75],  // (m)
            )
            .unwrap();
            println!("{df}");

            let mut file = File::create("data/test/output.csv").expect("could not create file");
            CsvWriter::new(&mut file)
                .include_header(true)
                .with_separator(b',')
                .finish(&mut df)
                .unwrap();

            let df_csv = CsvReadOptions::default()
                .with_has_header(true)
                .with_parse_options(CsvParseOptions::default().with_try_parse_dates(true))
                .try_into_reader_with_file_path(Some("data/test/output.csv".into()))
                .unwrap()
                .finish()
                .unwrap();
            println!("{df_csv}");

            let result = df
                .clone()
                .lazy()
                .select([
                    col("name"),
                    col("birthdate").dt().year().alias("birth_year"),
                    (col("weight") / col("height").pow(2)).alias("bmi"),
                ])
                .collect()
                .unwrap();
            println!("{result}");

            let result = df
                .clone()
                .lazy()
                .select([
                    col("name"),
                    (cols(["weight", "height"]).as_expr() * lit(0.95))
                        .round(2, RoundMode::default())
                        .name()
                        .suffix("-5%"),
                ])
                .collect()
                .unwrap();
            println!("{result}");
        }
    }
}
