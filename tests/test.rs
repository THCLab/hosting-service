use http::StatusCode;
use keri_witness_http::http_witness::HttpWitness;
use serde_json::Value;
use tempfile::tempdir;

#[tokio::test]
async fn basic_test() {
    tokio::spawn(async {
        let dir = tempdir().unwrap();
        let service = HttpWitness::new(&dir.path());
        service.listen(3030).await;
    });

    let sent_event = r#"{"v":"KERI10JSON00017e_","t":"icp","d":"ELYk-z-SuTIeDncLr6GhwVUKnv3n3F1bF18qkXNd2bpk","i":"ELYk-z-SuTIeDncLr6GhwVUKnv3n3F1bF18qkXNd2bpk","s":"0","kt":"2","k":["DSuhyBcPZEZLK-fcw5tzHn2N46wRCG_ZOoeKtWTOunRA","DVcuJOOJF1IE8svqEtrSuyQjGTd2HhfAkt9y2QkUtFJI","DT1iAhBWCkvChxNWsby2J0pJyxBIxbAtbLA0Ljx-Grh8"],"n":"E9izzBkXX76sqt0N-tfLzJeRqj0W56p4pDQ_ZqNCDpyw","bt":"0","b":[],"c":[],"a":[]}-AADAA39j08U7pcU66OPKsaPExhBuHsL5rO1Pjq5zMgt_X6jRbezevis6YBUg074ZNKAGdUwHLqvPX_kse4buuuSUpAQABphobpuQEZ6EhKLhBuwgJmIQu80ZUV1GhBL0Ht47Hsl1rJiMwE2yW7-yi8k3idw2ahlpgdd9ka9QOP9yQmMWGAQACM7yfK1b86p1H62gonh1C7MECDCFBkoH0NZRjHKAEHebvd2_LLz6cpCaqKWDhbM2Rq01f9pgyDTFNLJMxkC-fAQ"#;

    let client = reqwest::Client::new();
    let _res = client
        .post("http://localhost:3030/publish")
        .body(sent_event)
        .send()
        .await;

    let kel = client
        .get("http://localhost:3030/identifier/ELYk-z-SuTIeDncLr6GhwVUKnv3n3F1bF18qkXNd2bpk/kel")
        .send()
        .await;

    assert_eq!(kel.unwrap().text().await.unwrap(), sent_event);

    let receipts = client
        .get("http://localhost:3030/identifier/ELYk-z-SuTIeDncLr6GhwVUKnv3n3F1bF18qkXNd2bpk/receipts")
        .send()
        .await;

    assert!(receipts.is_ok());
    // println!("receipts = {}", receipts.unwrap().text().await.unwrap());
}

#[tokio::test]
async fn test_process_stream() {
    /// Helper function for checking expected response.
    /// # Arguments:
    /// * `parsed` - count of parsed events,
    /// * `not_parsed` - not parsed part of the stream,
    /// * `added` - count of events added to kel,
    /// * `errors` - count of events which processing failed
    /// * `response_str` - json string of the response
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

    let ok_stream = r#"{"v":"KERI10JSON000120_","t":"icp","d":"Et78eYkh8A3H9w6Q87EC5OcijiVEJT8KyNtEGdpPVWV8","i":"Et78eYkh8A3H9w6Q87EC5OcijiVEJT8KyNtEGdpPVWV8","s":"0","kt":"1","k":["DqI2cOZ06RwGNwCovYUWExmdKU983IasmUKMmZflvWdQ"],"n":"E7FuL3Z_KBgt_QAwuZi1lUFNC69wvyHSxnMFUsKjZHss","bt":"0","b":[],"c":[],"a":[]}-AABAAJEloPu7b4z8v1455StEJ1b7dMIz-P0tKJ_GBBCxQA8JEg0gm8qbS4TWGiHikLoZ2GtLA58l9dzIa2x_otJhoDA{"v":"KERI10JSON000155_","t":"rot","d":"EoU_JzojCvenHLPza5-K7z59yU7efQVrzciNdXoVDmlk","i":"Et78eYkh8A3H9w6Q87EC5OcijiVEJT8KyNtEGdpPVWV8","s":"1","p":"Et78eYkh8A3H9w6Q87EC5OcijiVEJT8KyNtEGdpPVWV8","kt":"1","k":["Dyb48eeVVXD7JAarHFAUffKcgYGvCQ4KWX00myzNLgzU"],"n":"ElBleBp2wS0n927E6W63imv-lRzU10uLYTRKzHNn19IQ","bt":"0","br":[],"ba":[],"a":[]}-AABAAXcEQQlT3id8LpTRDkFKVzF7n0d0w-3n__xgdf7rxTpAWUVsHthZcPtovCVr1kca1MD9QbfFAMpEtUZ02LTi3AQ{"v":"KERI10JSON000155_","t":"rot","d":"EYhzp9WCvSNFT2dVryQpVFiTzuWGbFNhVHNKCqAqBI8A","i":"Et78eYkh8A3H9w6Q87EC5OcijiVEJT8KyNtEGdpPVWV8","s":"2","p":"EoU_JzojCvenHLPza5-K7z59yU7efQVrzciNdXoVDmlk","kt":"1","k":["DyN13SKiF1FsVoVR5C4r_15JJLUBxBXBmkleD5AYWplc"],"n":"Em4tcl6gRcT2OLjbON4iz-fsw0iWQGBtwWic0dJY4Gzo","bt":"0","br":[],"ba":[],"a":[]}-AABAAZgqx0nZk4y2NyxPGypIloZikDzaZMw8EwjisexXwn-nr08jdILP6wvMOKZcxmCbAHJ4kHL_SIugdB-_tEvhBDg{"v":"KERI10JSON000155_","t":"rot","d":"EsL4LnyvTGBqdYC_Ute3ag4XYbu8PdCj70un885pMYpA","i":"Et78eYkh8A3H9w6Q87EC5OcijiVEJT8KyNtEGdpPVWV8","s":"3","p":"EYhzp9WCvSNFT2dVryQpVFiTzuWGbFNhVHNKCqAqBI8A","kt":"1","k":["DrcAz_gmDTuWIHn_mOQDeSK_aJIRiw5IMzPD7igzEDb0"],"n":"E_Y2NMHE0nqrTQLe57VPcM0razmxdxRVbljRCSetdjjI","bt":"0","br":[],"ba":[],"a":[]}-AABAAkk_Z4jS76LBiKrTs8tL32DNMndq5UQJ-NoteiTyOuMZfyP8jgxJQU7AiR7zWQZxzmiF0mT1JureItwDkPli5DA"#;
    // Expected parsing results
    let expected = (4, "", 4, 0);

    let res = client.post(url).body(ok_stream).send().await.unwrap();
    assert_eq!(res.status().clone(), StatusCode::OK);

    let response_str = res.text().await.unwrap();
    check_response(expected, &response_str);

    // Stream of 3 events with wrong signature of the last one
    let wrong_signature_stream = r#"{"v":"KERI10JSON000120_","t":"icp","d":"Ei8DuPp6MZStveFRUuhccKyHiXZnvkNZOUROppUBGXDk","i":"Dk1x3R6fUr1H9vMGutrTMkQsvWMTtljz5hf7fer1yaKs","s":"0","kt":"1","k":["Dk1x3R6fUr1H9vMGutrTMkQsvWMTtljz5hf7fer1yaKs"],"n":"Ed3DjyrBD2ShIQVJlE2_So-Wq7rABuidi_9F7ZIxoLdw","bt":"0","b":[],"c":[],"a":[]}-AABAAE91l4U5aEA8G_h-RV7HSQax-fHgvxe3vDKFwuLRR48Z4a05XBhYLHuSP5DqfyKUfq-KFmyesTw1JAsfzGzzCDA{"v":"KERI10JSON000155_","t":"rot","d":"EWn4NVvo-x5bEnRKQdSex0636ixOe1VA7Txhpy77F0UU","i":"Dk1x3R6fUr1H9vMGutrTMkQsvWMTtljz5hf7fer1yaKs","s":"1","p":"Ei8DuPp6MZStveFRUuhccKyHiXZnvkNZOUROppUBGXDk","kt":"1","k":["D4XwHtIF5SH84mnOELeNT8O0qSUpR7_3WPrI67Lv6vF8"],"n":"E0rSnZLuR7UE6v8ogR3XViXz6UoAOzfshh3Kxej4Wgow","bt":"0","br":[],"ba":[],"a":[]}-AABAA64GKfvBUSfYeIA3X9_YlCJ399Dpbazu3afoBrxrcwPie9TuxS1FHQdhJsH8TYvSGMv5oXnAOTJtj0H-OnhVuBA{"v":"KERI10JSON0000cb_","t":"ixn","d":"EtaGH377ijB36vDzvIHnLNVaF61wZ6OTBSorwiQVth10","i":"Dk1x3R6fUr1H9vMGutrTMkQsvWMTtljz5hf7fer1yaKs","s":"2","p":"EWn4NVvo-x5bEnRKQdSex0636ixOe1VA7Txhpy77F0UU","a":[]}-AABAAw2T9EgLv1tFCNhF1GxDYQcUyq2jO2Q-15YoZFMS9TJqax06bjnYAVTCRUR9p-S6tKlEd0SDoLfndXdKgXMUBAg"#;
    let expected = (3, "", 2, 1);

    let res = client
        .post(url)
        .body(wrong_signature_stream)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status().clone(), StatusCode::OK);
    let response_str = res.text().await.unwrap();
    check_response(expected, &response_str);

    // Stream of 4 events where last event is duplicate
    let dup_stream = r#"{"v":"KERI10JSON000120_","t":"icp","d":"EhLxbZ-vgkh0BceM7wN0W-3csdpNFixtmF_iXWSFhfOc","i":"DMJ7KB0AI7PV_cCxYCylCilz8Sgy5Sht_bKdnr-zGZIg","s":"0","kt":"1","k":["DMJ7KB0AI7PV_cCxYCylCilz8Sgy5Sht_bKdnr-zGZIg"],"n":"EY8ZQRFzVHB0tl4mOQda4YFNlEo8zqz07R96tSdb5T78","bt":"0","b":[],"c":[],"a":[]}-AABAAFkFnXnXN6VRh0St4YNOsH7ymhotbO9cxQ6-xXMJnBUpn5GnfHzb73CVlmpVdusRlLKw1sl8OXuC5uOpwaMKCCQ{"v":"KERI10JSON000155_","t":"rot","d":"Eu3JKxrsA_5dDVO_ha7GgsdWzI5AaLZrMpg00bMoeChc","i":"DMJ7KB0AI7PV_cCxYCylCilz8Sgy5Sht_bKdnr-zGZIg","s":"1","p":"EhLxbZ-vgkh0BceM7wN0W-3csdpNFixtmF_iXWSFhfOc","kt":"1","k":["DB2fHkRv48L2-qUF9RMGQWLrslLx9cV8R5WGZW2jmyAY"],"n":"E0TRM5EgwKkjxMx5SCJ0SdQHhnHxIZEvRVWjHjL3HH58","bt":"0","br":[],"ba":[],"a":[]}-AABAA8y4N7YzttcabuZWWt9TA-RoPcvPabNX_VNCjeFqelknBER0--ZBaTrfM5MCFzeS4qBz5oLwnhZTK-iS-SE5hDg{"v":"KERI10JSON0000cb_","t":"ixn","d":"EVY9AZvPYsPHdaVgZc8hFMuZj6XCji8QYJUmMiPSPFrg","i":"DMJ7KB0AI7PV_cCxYCylCilz8Sgy5Sht_bKdnr-zGZIg","s":"2","p":"Eu3JKxrsA_5dDVO_ha7GgsdWzI5AaLZrMpg00bMoeChc","a":[]}-AABAATJy0kkG76iEAPuzhkIcziBnQ9OAR9E8OAmcd3wfY6ACurmMt_SN5iuZX5nYf_qlIf_SwMiT7tvV4TsNwjAT6Cg{"v":"KERI10JSON0000cb_","t":"ixn","d":"EVY9AZvPYsPHdaVgZc8hFMuZj6XCji8QYJUmMiPSPFrg","i":"DMJ7KB0AI7PV_cCxYCylCilz8Sgy5Sht_bKdnr-zGZIg","s":"2","p":"Eu3JKxrsA_5dDVO_ha7GgsdWzI5AaLZrMpg00bMoeChc","a":[]}-AABAATJy0kkG76iEAPuzhkIcziBnQ9OAR9E8OAmcd3wfY6ACurmMt_SN5iuZX5nYf_qlIf_SwMiT7tvV4TsNwjAT6Cg"#;
    let expected = (4, "", 3, 1);
    let res = client.post(url).body(dup_stream).send().await.unwrap();
    assert_eq!(res.status().clone(), StatusCode::OK);
    let response_str = res.text().await.unwrap();
    check_response(expected, &response_str);

    // event with missing signatures
    let missing_sigs_stream = r#"{"v":"KERI10JSON000120_","t":"icp","d":"E45c-zJTVESfLf3IPIEaUT7C5IPrzNQxeMApFfi7BKTg","i":"DmlrLhWNRHnMIYCIA0dr6ZIJyG5GMdgnoPg4Ov6uNwJ8","s":"0","kt":"1","k":["DmlrLhWNRHnMIYCIA0dr6ZIJyG5GMdgnoPg4Ov6uNwJ8"],"n":"EjPWs9__ZD_dIZAgnzIt3NgDZ7b7FYUD_agzmuscNUJE","bt":"0","b":[],"c":[],"a":[]}"#;

    let expected = (1, "", 0, 1);
    let res = client
        .post(url)
        .body(missing_sigs_stream)
        .send()
        .await
        .unwrap();
    let response_str = res.text().await.unwrap();
    check_response(expected, &response_str);

    // Any string.
    let just_str = r#"no events here"#;
    let res = client.post(url).body(just_str).send().await.unwrap();

    assert_eq!(res.status().clone(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        res.text().await.unwrap(),
        "{\"error\":\"Stream can't be parsed\"}"
    );
}
