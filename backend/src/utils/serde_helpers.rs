// serde 反序列化辅助函数

use serde::Deserializer;

/// 自定义反序列化函数：支持单个值、逗号分隔字符串或数组
/// 用于处理查询参数中的数组字段
/// 支持格式: courseSns=1,2,3 或 courseSns=1 或 courseSns[]=1&courseSns[]=2
pub fn deserialize_vec_i64<'de, D>(deserializer: D) -> Result<Vec<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{Error, SeqAccess, Unexpected, Visitor};
    use std::fmt;

    struct VecI64Visitor;

    impl<'de> Visitor<'de> for VecI64Visitor {
        type Value = Vec<i64>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a single value, comma-separated string, or an array of i64 values")
        }

        // 处理单个字符串值（可能是单个数字或逗号分隔）
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            // 先尝试按逗号分割
            if value.contains(',') {
                let mut result = Vec::new();
                for part in value.split(',') {
                    let trimmed = part.trim();
                    if !trimmed.is_empty() {
                        let num = trimmed.parse::<i64>().map_err(|_| {
                            E::invalid_value(Unexpected::Str(value), &"comma-separated i64 numbers")
                        })?;
                        result.push(num);
                    }
                }
                Ok(result)
            } else {
                // 单个值
                let num = value
                    .parse::<i64>()
                    .map_err(|_| E::invalid_value(Unexpected::Str(value), &"a valid i64 number"))?;
                Ok(vec![num])
            }
        }

        // 处理单个整数（如 courseSns=1）
        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(vec![value])
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(vec![value as i64])
        }

        // 处理字符串序列（如 courseSns[]=1&courseSns[]=2）
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut result = Vec::new();
            while let Some(s) = seq.next_element::<String>()? {
                // 每个元素可能是逗号分隔的
                if s.contains(',') {
                    for part in s.split(',') {
                        let trimmed = part.trim();
                        if !trimmed.is_empty() {
                            let num = trimmed.parse::<i64>().map_err(|_| {
                                A::Error::invalid_value(Unexpected::Str(&s), &"a valid i64 number")
                            })?;
                            result.push(num);
                        }
                    }
                } else {
                    let num = s.parse::<i64>().map_err(|_| {
                        A::Error::invalid_value(Unexpected::Str(&s), &"a valid i64 number")
                    })?;
                    result.push(num);
                }
            }
            Ok(result)
        }
    }

    deserializer.deserialize_any(VecI64Visitor)
}
