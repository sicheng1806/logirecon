use std::collections::HashMap;

pub trait AsHeaders {
    fn as_headers(&self) -> HashMap<String, String>;
    fn update_headers(&mut self, headers: impl IntoIterator<Item = (String, String)>);
}

#[macro_export]
macro_rules! define_headers {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident [
            $(
                $(#[$field_meta:meta])*
                $field:ident: $label:literal
            ),+
            $(,)?
        ]
    ) => {
        $(#[$meta])*
        $vis struct $name {
            $(
                $(#[$field_meta])*
                pub $field: String
            ),+
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    $(
                        $field: $label.to_string()
                    ),+
                }
            }
        }

        impl $crate::parser::AsHeaders for $name {
            fn as_headers(&self) -> std::collections::HashMap<String, String> {
                std::collections::HashMap::from([
                    $(
                        ($label.to_string(), self.$field.clone())
                    ),+
                ])
            }
            fn update_headers(&mut self, headers: impl IntoIterator<Item = (String, String)>) {
                use std::collections::HashMap;
                let map: HashMap<String, String> = HashMap::from_iter(headers.into_iter());
                $(
                    if let Some(v) = map.get($label) {
                        self.$field = v.clone();
                    }
                )+
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_macro() {
        define_headers! {
            /// 嘀嗒嘀表头
            #[derive(Debug, Clone)]
            pub struct DDDHeaders [
                /// 日期
                date: "签入日期",
                /// 运单号
                waybill_no: "运单号",
                /// 订单号
                shipment_no: "FBA单号",
                /// 仓库编码
                warehouse_code: "目的仓",
                /// 件数
                n_pieces: "件数",
                /// 收费重
                chargeable_weight: "收费重",
                /// 计算公式
                formula: "计算公式"
            ]
        }

        let mut headers = DDDHeaders::default();
        println!("{:?}", headers);
        println!("{:?}", headers.as_headers());
        headers
            .update_headers([("运单号", "FBA单号")].map(|(a, b)| (a.to_string(), b.to_string())));
        println!("{:?}", headers);
    }
}
