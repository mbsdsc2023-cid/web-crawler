#[tokio::test]
async fn test() {
    use crate::crawler::Crawler;

    let crawler = match Crawler::new(
        r"MBSD\{[0-9a-zA-Z]+\}",
        "https://mbsdsc2023-cid.github.io/sample-site",
    ) {
        Ok(c) => c,
        Err(e) => panic!("{}: {:?}", e, e),
    };

    let res = match crawler.execute().await {
        Ok(res) => res,
        Err(e) => panic!("{}: {:?}", e, e),
    };

    // TODO
    assert_eq!(res, vec![])
}
