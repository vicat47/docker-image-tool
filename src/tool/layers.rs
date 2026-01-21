//! 层集合操作模块。
//!
//! 提供 Docker 镜像层集合的类型定义和常用操作。

use std::collections::HashSet;

/// 层集合类型别名。
pub type LayerSet = HashSet<String>;

/// 计算两个层集合的差集（`a - b`）。
pub fn difference(a: &LayerSet, b: &LayerSet) -> LayerSet {
    a.difference(b).cloned().collect()
}

/// 从字符串切片创建层集合。
pub fn from_slice(layers: &[&str]) -> LayerSet {
    layers.iter().map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difference() {
        let a: LayerSet = vec!["layer1".to_string(), "layer2".to_string()].into_iter().collect();
        let b: LayerSet = vec!["layer2".to_string(), "layer3".to_string()].into_iter().collect();
        let diff = difference(&a, &b);
        assert_eq!(diff.len(), 1);
        assert!(diff.contains("layer1"));
    }
}