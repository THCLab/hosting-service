use http::StatusCode;
use keri_witness_http::http_witness::HttpWitness;
use serde_json::Value;
use tempfile::tempdir;

#[tokio::test]
async fn test_process() {
    tokio::spawn(async {
        let dir = tempdir().unwrap();
        let service = HttpWitness::new(&dir.path());
        service.listen(3030).await;
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

#[tokio::test]
async fn test_process_stream() {
    /// Helper function for checking expected response.
    /// # Arguments:
    /// * `parsed` - count of parsed events,
    /// * `not_parsed` - not parsed part of the stream,
    /// * `added` - count of events added to kel,
    /// * `errors` - count of events which processing failed
    /// * `response_str`
    fn check_response(
        (parsed, not_parsed, added, errors): (u64, &str, u64, u64),
        response_str: &str,
    ) {
        // Parse the string of data into serde_json::Value.
        let v: Value = serde_json::from_str(&response_str).unwrap();
        assert_eq!(v["parsed"], parsed);
        assert_eq!(v["not_parsed"], not_parsed);
        assert_eq!(
            if let Value::Array(ar) = &v["receipts"] {
                ar.len() as u64
            } else {
                0
            },
            added
        );
        assert_eq!(
            if let Value::Array(ar) = &v["errors"] {
                ar.len() as u64
            } else {
                0
            },
            errors
        );
    }

    tokio::spawn(async {
        let dir = tempdir().unwrap();
        let service = HttpWitness::new(&dir.path());
        service.listen(3031).await;
    });
    let client = reqwest::Client::new();
    let url = "http://localhost:3031/publish";

    let ok_stream = r#"{"v":"KERI10JSON0000ed_","i":"D9LusyVm8vgm4a7CSMbFUHTFuxmkTFBK87UYmqrKib6k","s":"0","t":"icp","kt":"1","k":["D9LusyVm8vgm4a7CSMbFUHTFuxmkTFBK87UYmqrKib6k"],"n":"EoZ9wWAp8ZZFcBkBinYc_CNlcJ3T_SNO0tQaG5FbZWBU","bt":"0","b":[],"c":[],"a":[]}-AABAAvN4aOvZcI2luquny-83fXxJ81qbwNhDXj83B0-GFy-tZ6v49rm_-tnjGDxbSrowiFuXltMy69IjtJinLU5XlAA{"v":"KERI10JSON000122_","i":"D9LusyVm8vgm4a7CSMbFUHTFuxmkTFBK87UYmqrKib6k","s":"1","t":"rot","p":"EBG9-3y2tJ3ro6W_gb1fwH2OHQtuGPZJTwYxuao_RErk","kt":"1","k":["DGWesjt1NoxNPnJhwNPIhbfi1jI-eJFfZsu3JtX-t51U"],"n":"EUhUsYOPK6kC1Eg85Z0w0FUNRK3YHk0mt6Awjx5EpjmQ","bt":"0","br":[],"ba":[],"a":[]}-AABAAWujbSavnnRcncfSewbf6sRa6c5phcGFi3Uy0YqYuDe39nkgH5zFVHlCC-GK_aii7WzX3kSnw-TjEaxsugG4NCw{"v":"KERI10JSON000098_","i":"D9LusyVm8vgm4a7CSMbFUHTFuxmkTFBK87UYmqrKib6k","s":"2","t":"ixn","p":"EQDQl9ncRKIOb7VZIVF5DJCSpcArsiOjmkAf9mzv4Bjo","a":[]}-AABAA1uqYwzDr85Teh7h_BsO2pULNGQQezNVzb_UrM3CrbLShOLHDs-2290sm0z2qT7c4OU7YOAoUjnlDQal6DUw9AQ"#;
    // Expected parsing results
    let expected = (3, "", 3, 0);

    let res = client
        .post(url)
        .body(ok_stream)
        .send()
        .await.unwrap();
    assert_eq!(res.status().clone(), StatusCode::OK);

    let response_str = res.text().await.unwrap();
    check_response(expected, &response_str);

    // Stream of 3 events with wrong signature of the last one
    let wrong_signature_stream = r#"{"v":"KERI10JSON0000ed_","i":"DZBLBkuS9WyEuzdCfYaR_hx9v0rbwEHJs43gQiAd13Ik","s":"0","t":"icp","kt":"1","k":["DZBLBkuS9WyEuzdCfYaR_hx9v0rbwEHJs43gQiAd13Ik"],"n":"E6NFyW73Skfw9eXyX7onBaVtZBqflSck_Qhy6khkf7A0","bt":"0","b":[],"c":[],"a":[]}-AABAApj4Ja0u6BsdXuiV-A9zo0PkcHXx9qf4rx6_Ox74dWeAuaYaEbwAbSZ4K3RAfyfidfsxdMuAqh9f9M6xqoOtCAw{"v":"KERI10JSON000122_","i":"DZBLBkuS9WyEuzdCfYaR_hx9v0rbwEHJs43gQiAd13Ik","s":"1","t":"rot","p":"EkxPH2uHog4DA1tubXgzj6pvxkZGCgs1RnlgTxsU_pGw","kt":"1","k":["DLQZYcd3D_obmvmfCTp6BS5Ej_caeaTIMIsjZrweLYp0"],"n":"EQtjUVpj4zkpOK-TG9W-RIILFK-XYiQCKEtG0yRTxKXo","bt":"0","br":[],"ba":[],"a":[]}-AABAAZFAvuigM8kYaHhXPo2Ud15r3eEaM_YtXabgHCZdKeIM5ONGPeXAIGIXAl0Mv-3qb4BPaDQCZdWcKOmrbm0JkBA{"v":"KERI10JSON000098_","i":"DZBLBkuS9WyEuzdCfYaR_hx9v0rbwEHJs43gQiAd13Ik","s":"2","t":"ixn","p":"EcnEEIq3loiJNwQ9Zdc0Okfb2ovFMl5EFudN0g5BoyLY","a":[]}-AABAAGGK3bN91NLhd3AVxYicIL4k4OBZEwiNUnySo5MlOsoNtqSTqkTZllujE8QnHFfu5x62q2meaDf-rvCNKbAltCw"#;
    let expected = (3, "", 2, 1);

    let res = client
        .post(url)
        .body(wrong_signature_stream)
        .send()
        .await.unwrap();
    assert_eq!(res.status().clone(), StatusCode::OK);
    let response_str = res.text().await.unwrap();
    check_response(expected, &response_str);

    // Stream of 4 events where last event is duplicate
    let dup_stream = r#"{"v":"KERI10JSON0000ed_","i":"D15_ZODok1AuPrGfzE6VhMmT9gn6f7beC2tGIduPxRo8","s":"0","t":"icp","kt":"1","k":["D15_ZODok1AuPrGfzE6VhMmT9gn6f7beC2tGIduPxRo8"],"n":"EvBQ0CPWOANaC_SX4nPejD4tAQ5G_dvp1xR-W3zMbM-U","bt":"0","b":[],"c":[],"a":[]}-AABAA8RV41XkhnTwvVx-eF8tAFYywAmD_V0TRiRasqS_kYnG3F9jScQpCTteHOAuMpoY9hfJN8RvudooqPrvpJaGoBA{"v":"KERI10JSON000122_","i":"D15_ZODok1AuPrGfzE6VhMmT9gn6f7beC2tGIduPxRo8","s":"1","t":"rot","p":"E7kU_Im8oggF7yGCIb2hFqtJxub5dao6DmNBio1Ra73I","kt":"1","k":["DxauUsjTBfpGLKzpr7swM2gpxNQGZwjQbkB-vUASKrYo"],"n":"EZUJTljHFepUZgkhtd3Fzuu0k8pyk2uFx1Zp3IizNL14","bt":"0","br":[],"ba":[],"a":[]}-AABAAAP-vmtkjB0VVaaXDeLSzNxmT--widHi3MNn7m58ecqUMzgu_Oxp0uRKUt4dDyetWV0NwSKYq3KWnybS46guaAA{"v":"KERI10JSON000098_","i":"D15_ZODok1AuPrGfzE6VhMmT9gn6f7beC2tGIduPxRo8","s":"2","t":"ixn","p":"EW9ikln3dOaJ1d7FY0AcCbzB-VU4HRJg2GOELVFosXBI","a":[]}-AABAAoQwEjNqR7Lrfh-coWJezWw_frdGr8IRqnig2_-q9hELAdvtFfrD6IgsikoIWKNSyzYuzGtwQtKO-H3Y3dMc9CQ{"v":"KERI10JSON000122_","i":"D15_ZODok1AuPrGfzE6VhMmT9gn6f7beC2tGIduPxRo8","s":"1","t":"rot","p":"E7kU_Im8oggF7yGCIb2hFqtJxub5dao6DmNBio1Ra73I","kt":"1","k":["DxauUsjTBfpGLKzpr7swM2gpxNQGZwjQbkB-vUASKrYo"],"n":"EZUJTljHFepUZgkhtd3Fzuu0k8pyk2uFx1Zp3IizNL14","bt":"0","br":[],"ba":[],"a":[]}-AABAAAP-vmtkjB0VVaaXDeLSzNxmT--widHi3MNn7m58ecqUMzgu_Oxp0uRKUt4dDyetWV0NwSKYq3KWnybS46guaAA"#;
    let expected = (4, "", 3, 1);
    let res = client
        .post(url)
        .body(dup_stream)
        .send()
        .await.unwrap();
    assert_eq!(res.status().clone(), StatusCode::OK);
    let response_str = res.text().await.unwrap();
    check_response(expected, &response_str);

    // event with missing signatures
    let missing_sigs_stream = r#"{"v":"KERI10JSON0000ed_","i":"DSuhyBcPZEZLK-fcw5tzHn2N46wRCG_ZOoeKtWTOunRA","s":"0","t":"icp","kt":"1","k":["DSuhyBcPZEZLK-fcw5tzHn2N46wRCG_ZOoeKtWTOunRA"],"n":"EPYuj8mq_PYYsoBKkzX1kxSPGYBWaIya3slgCOyOtlqU","bt":"0","b":[],"c":[],"a":[]}"#;

    let res = client
        .post(url)
        .body(missing_sigs_stream)
        .send()
        .await
        .unwrap();

    assert_eq!(res.status().clone(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(res.text().await.unwrap(), "{\"error\":\"Stream can't be parsed\"}");


    // Any string.
    let just_str = r#"no events here"#;
    let res = client
        .post(url)
        .body(just_str)
        .send()
        .await
        .unwrap();
    
    assert_eq!(res.status().clone(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(res.text().await.unwrap(), "{\"error\":\"Stream can't be parsed\"}");

}
