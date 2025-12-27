#[cfg(test)]
mod tests {
    use rp::inventory::{InnerInventory, Pool, Resource};
    use serde_saphyr::from_str;
    use std::collections::HashMap;
    use std::sync::Weak;

    fn build_simple_inventory() -> InnerInventory {
        InnerInventory {
            pools: vec![Pool {
                name: "pool1".into(),
                attributes: vec!["attr1".into(), "attr2".into()],
                location: "location1".into(),
                resources: vec![
                    Resource {
                        attributes: vec!["RA1".into(), "RA2".into()],
                        properties: HashMap::new(),
                    },
                    Resource {
                        attributes: vec!["RB1".into(), "RB2".into()],
                        properties: HashMap::new(),
                    },
                ],
                user: Weak::new(),
            }],
        }
    }
    #[test]
    fn test_simple_config() {
        let expected = build_simple_inventory();

        let yaml_input = r#"
pools:
  - name: pool1
    attributes: [attr1, attr2]
    location: location1
    resources:
      - attributes: [RA1, RA2]
        properties: {}
      - attributes: [RB1, RB2]
        properties: {}
    user: None
"#;
        let parsed = from_str(yaml_input).unwrap();
        assert_eq!(expected, parsed);
    }
}
