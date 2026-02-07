#[cfg(test)]
mod tests {
    use rp::inventory::{
        ClientResourceRequest, Inventory, InventoryManager, LocalRespoClient,
        LocalRespoClientFactory, Pool, Resource, ResourceRequest, ResourceRequestError,
    };
    use std::collections::HashMap;
    use std::sync::Weak;
    use tokio::join;
    use tokio::time::{Duration, sleep};

    fn build_simple_inventory_manager() -> InventoryManager {
        InventoryManager::new(Inventory {
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
        })
    }
    fn build_simple_clientfactory() -> LocalRespoClientFactory {
        let inventory_manager = build_simple_inventory_manager();
        LocalRespoClientFactory::new(inventory_manager)
    }
    fn build_ok_request() -> ResourceRequest {
        ResourceRequest {
            pool_attributes: Some(vec!["attr1".into()]),
            ..Default::default()
        }
    }
    fn build_simple_client() -> LocalRespoClient {
        let clientfactory = build_simple_clientfactory();
        clientfactory.create("client_a".into())
    }
    #[tokio::test]
    async fn test_by_name_positive() {
        let mut client = build_simple_client();
        let ok_request = ResourceRequest {
            by_name: Some("pool1".into()),
            ..Default::default()
        };
        assert!(client.request(&ok_request).await.is_ok());
    }
    #[tokio::test]
    async fn test_by_name_negative() {
        let mut client = build_simple_client();
        let nok_request = ResourceRequest {
            by_name: Some("pool_not_there".into()),
            ..Default::default()
        };
        assert!(matches!(
            client.request(&nok_request).await,
            Err(ResourceRequestError::Impossible)
        ));
    }

    #[tokio::test]
    async fn test_ok_pool_attributes() {
        let mut client = build_simple_client();
        let ok_request = build_ok_request();
        assert!(client.request(&ok_request).await.is_ok());
    }

    #[tokio::test]
    async fn test_nok_pool_attributes() {
        let mut client = build_simple_client();
        let ok_request = build_ok_request();
        let nok_request = ResourceRequest {
            pool_attributes: Some(vec!["attr3".into()]),
            ..ok_request.clone()
        };
        assert!(matches!(
            client.request(&nok_request).await,
            Err(ResourceRequestError::Impossible)
        ));
    }

    #[tokio::test]
    async fn test_nok_location() {
        let mut client = build_simple_client();
        let ok_request = build_ok_request();
        let nok_request = ResourceRequest {
            location: Some("abroad".into()),
            ..ok_request.clone()
        };
        assert!(matches!(
            client.request(&nok_request).await,
            Err(ResourceRequestError::Impossible)
        ));
    }
    #[tokio::test]
    async fn test_resource_attributes_match() {
        let mut client = build_simple_client();
        let ok_request = build_ok_request();
        let ra_ok_request = ResourceRequest {
            resource_attributes: Some(vec![vec!["RA1".into()]]),
            ..ok_request.clone()
        };
        assert!(client.request(&ra_ok_request.clone()).await.is_ok());
    }
    #[tokio::test]
    async fn test_resource_attributes_mismatch() {
        let mut client = build_simple_client();
        let ok_request = build_ok_request();
        // Failure case
        let nok_request = ResourceRequest {
            resource_attributes: Some(vec![vec!["RA3".into()]]),
            ..ok_request.clone()
        };
        let result = client.request(&nok_request).await;
        assert!(
            matches!(result.as_ref(), Err(ResourceRequestError::Impossible)),
            "Unexpected error: {:?}",
            result.as_ref().unwrap_err()
        );
    }
    #[tokio::test]
    async fn test_concurrent_usage_returns_error() {
        let clientfactory = build_simple_clientfactory();
        let ok_request = build_ok_request();

        let mut client_a = clientfactory.create("client_a".into());
        let mut client_b = clientfactory.create("client_b".into());

        join!(
            async {
                assert!(client_a.request(&ok_request.clone()).await.is_ok());
                sleep(Duration::from_secs(1)).await;
            },
            async {
                sleep(Duration::from_millis(100)).await;
                assert!(matches!(
                    client_b.request(&ok_request.clone()).await,
                    Err(ResourceRequestError::InUse)
                ));
            }
        );
    }
    #[tokio::test]
    async fn test_concurrent_timeout() {
        let clientfactory = build_simple_clientfactory();
        let ok_request = build_ok_request();
        let ok_with_timeout = ResourceRequest {
            timeout: Some(Duration::from_millis(500)),
            ..ok_request.clone()
        };

        let mut client_a = clientfactory.create("client_a".into());
        let mut client_b = clientfactory.create("client_b".into());

        join!(
            async move {
                assert!(client_a.request(&ok_request).await.is_ok());
                sleep(Duration::from_secs(1)).await;
            },
            async move {
                sleep(Duration::from_millis(100)).await;
                assert!(matches!(
                    client_b.request(&ok_with_timeout).await,
                    Err(ResourceRequestError::TimeOut)
                ));
            }
        );
    }

    #[tokio::test]
    async fn test_concurrent_becomes_available() {
        // FIXME: test should be using process time instead of walltime
        let clientfactory = build_simple_clientfactory();
        let ok_request = build_ok_request();
        let ok_with_timeout = ResourceRequest {
            timeout: Some(Duration::from_millis(1000)),
            ..ok_request.clone()
        };

        let mut client_a = clientfactory.create("client_a".into());
        let mut client_b = clientfactory.create("client_b".into());

        join!(
            async move {
                let lease = client_a.request(&ok_request).await;
                assert!(lease.is_ok());
                sleep(Duration::from_millis(100)).await;
            },
            async move {
                sleep(Duration::from_millis(100)).await;
                let result = client_b.request(&ok_with_timeout).await;
                assert!(
                    result.is_ok(),
                    "Unexpected error: {:?}",
                    result.unwrap_err()
                );
            }
        );
    }
}
