use keri_witness_http::http_witness::HttpWitness;
use tempfile::tempdir;

#[tokio::test]
async fn test_process() {
    tokio::spawn(async {
        let dir = tempdir().unwrap();
        let service = HttpWitness::new(&dir.path());
        service.listen().await;
    });

    let sent_event = r#"{"v":"KERI10JSON0000ed_","i":"DSuhyBcPZEZLK-fcw5tzHn2N46wRCG_ZOoeKtWTOunRA","s":"0","t":"icp","kt":"1","k":["DSuhyBcPZEZLK-fcw5tzHn2N46wRCG_ZOoeKtWTOunRA"],"n":"EPYuj8mq_PYYsoBKkzX1kxSPGYBWaIya3slgCOyOtlqU","bt":"0","b":[],"c":[],"a":[]}-AABAAmagesCSY8QhYYHCJXEWpsGD62qoLt2uyT0_Mq5lZPR88JyS5UrwFKFdcjPqyKc_SKaKDJhkGWCk07k_kVkjyCA"#;

    let client = reqwest::Client::new();
    let _res = client
        .post("http://localhost:3030/publish")
        .body(sent_event)
        .send()
        .await;

    let kel = client
        .get("http://localhost:3030/identifier/DSuhyBcPZEZLK-fcw5tzHn2N46wRCG_ZOoeKtWTOunRA/kel")
        .send()
        .await;

    assert_eq!(kel.unwrap().text().await.unwrap(), sent_event);

    let receipts = client
        .get("http://localhost:3030/identifier/DSuhyBcPZEZLK-fcw5tzHn2N46wRCG_ZOoeKtWTOunRA/receipts")
        .send()
        .await;

    assert!(receipts.is_ok());
    println!("receipts = {}", receipts.unwrap().text().await.unwrap());
}
