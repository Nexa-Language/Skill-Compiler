//! Nested Data Depth Computation (IR-level)
//!
//! Pure computation function for determining the maximum nesting depth
//! of a JSON Value. Used by NestedDataDetector in the Analyzer phase
//! to decide whether YAML optimization is required.
//!
//! Academic basis:
//! - YAML nested data accuracy: 51.9% [48.8%, 55.0%]
//! - Markdown accuracy: 48.2% [45.1%, 51.3%]
//! - JSON accuracy: 43.1% [40.1%, 46.2%]

/// Compute the maximum nesting depth of a `serde_json::Value`.
///
/// Depth is defined as the longest chain of nested `Object` or `Array`
/// containers from root to leaf. Primitive values (string, number, bool, null)
/// contribute depth 0.
///
/// # Examples
///
/// ```
/// use serde_json::json;
/// use nexa_skill_core::ir::compute_nested_depth;
///
/// // Flat object → depth 1
/// let flat = json!({"a": 1, "b": 2});
/// assert_eq!(compute_nested_depth(&flat), 1);
///
/// // Nested object → depth 3
/// let nested = json!({"a": {"b": {"c": 1}}});
/// assert_eq!(compute_nested_depth(&nested), 3);
///
/// // Primitive → depth 0
/// let prim = json!(42);
/// assert_eq!(compute_nested_depth(&prim), 0);
/// ```
pub fn compute_nested_depth(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Object(map) => {
            if map.is_empty() {
                1
            } else {
                1 + map
                    .values()
                    .map(compute_nested_depth)
                    .max()
                    .unwrap_or(0)
            }
        }
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                1
            } else {
                1 + arr
                    .iter()
                    .map(compute_nested_depth)
                    .max()
                    .unwrap_or(0)
            }
        }
        // Primitives: string, number, bool, null → depth 0
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_primitive_depth() {
        assert_eq!(compute_nested_depth(&json!(42)), 0);
        assert_eq!(compute_nested_depth(&json!("hello")), 0);
        assert_eq!(compute_nested_depth(&json!(true)), 0);
        assert_eq!(compute_nested_depth(&json!(null)), 0);
    }

    #[test]
    fn test_flat_object_depth() {
        let flat = json!({"a": 1, "b": 2, "c": 3});
        assert_eq!(compute_nested_depth(&flat), 1);
    }

    #[test]
    fn test_flat_array_depth() {
        let arr = json!([1, 2, 3]);
        assert_eq!(compute_nested_depth(&arr), 1);
    }

    #[test]
    fn test_empty_object_depth() {
        assert_eq!(compute_nested_depth(&json!({})), 1);
    }

    #[test]
    fn test_empty_array_depth() {
        assert_eq!(compute_nested_depth(&json!([])), 1);
    }

    #[test]
    fn test_nested_object_depth_2() {
        let nested = json!({"a": {"b": 1}});
        assert_eq!(compute_nested_depth(&nested), 2);
    }

    #[test]
    fn test_nested_object_depth_3() {
        let nested = json!({"a": {"b": {"c": 1}}});
        assert_eq!(compute_nested_depth(&nested), 3);
    }

    #[test]
    fn test_deeply_nested_depth_5() {
        let nested = json!({"l1": {"l2": {"l3": {"l4": {"l5": "deep"}}}}});
        assert_eq!(compute_nested_depth(&nested), 5);
    }

    #[test]
    fn test_mixed_object_and_array() {
        // Object containing an array of objects
        let mixed = json!({
            "users": [
                {"name": "Alice", "address": {"city": "NYC"}},
                {"name": "Bob", "address": {"city": "LA"}}
            ]
        });
        // depth: root(1) → "users"(1) → array(1) → object(1) → "address"(1) → object(1) = 5
        // But actually: root Object(1) → array(1) → object(1) → object(1) = 4
        // Let's trace: root=Object(1 + max(children))
        //   "users" value = Array(1 + max(children))
        //     each element = Object(1 + max(children))
        //       "name" = 0, "address" = Object(1 + max(children))
        //         "city" = 0 → depth 1
        //       → depth 2
        //     → depth 3
        //   → depth 4
        assert_eq!(compute_nested_depth(&mixed), 4);
    }

    #[test]
    fn test_max_depth_across_siblings() {
        // Object with two keys: one shallow, one deep
        let siblings = json!({
            "shallow": 1,
            "deep": {"a": {"b": {"c": 1}}}
        });
        // max(shallow=0, deep=3) → 1 + 3 = 4
        assert_eq!(compute_nested_depth(&siblings), 4);
    }
}