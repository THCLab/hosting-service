# hosting-service

### Usage example

`cargo run`

* to publish event:

    ```curl -X POST -d '{"v":"KERI10JSON0000ed_","i":"DSuhyBcPZEZLK-fcw5tzHn2N46wRCG_ZOoeKtWTOunRA","s":"0","t":"icp","kt":"1","k":["DSuhyBcPZEZLK-fcw5tzHn2N46wRCG_ZOoeKtWTOunRA"],"n":"EPYuj8mq_PYYsoBKkzX1kxSPGYBWaIya3slgCOyOtlqU","bt":"0","b":[],"c":[],"a":[]}-AABAAmagesCSY8QhYYHCJXEWpsGD62qoLt2uyT0_Mq5lZPR88JyS5UrwFKFdcjPqyKc_SKaKDJhkGWCk07k_kVkjyCA' http://localhost:3030/publish```
    
* to get kel of `DSuhyBcPZEZLK-fcw5tzHn2N46wRCG_ZOoeKtWTOunRA`: 

    ```http://localhost:3030/kel/DSuhyBcPZEZLK-fcw5tzHn2N46wRCG_ZOoeKtWTOunRA```

* to get receipts for `DSuhyBcPZEZLK-fcw5tzHn2N46wRCG_ZOoeKtWTOunRA`: 

    ```http://localhost:3030/receipts/DSuhyBcPZEZLK-fcw5tzHn2N46wRCG_ZOoeKtWTOunRA```