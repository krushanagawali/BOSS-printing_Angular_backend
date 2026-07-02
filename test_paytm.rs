use reqwest;
use serde::{Deserialize, Serialize};
use hmac::{Hmac, Mac, NewMac};
use sha2::Sha256;
use hex;

#[derive(Serialize)]
struct PaytmInitiateBody {
    requestType: String,
    mid: String,
    websiteName: String,
    orderId: String,
    callbackUrl: String,
    txnAmount: PaytmTxnAmount,
    userInfo: PaytmUserInfo,
    channelId: String,
    industryTypeId: String,
}

#[derive(Serialize)]
struct PaytmTxnAmount {
    value: String,
    currency: String,
}

#[derive(Serialize)]
struct PaytmUserInfo {
    custId: String,
}

#[derive(Serialize)]
struct PaytmHead {
    signature: String,
}

#[derive(Serialize)]
struct PaytmInitiateRequest {
    body: PaytmInitiateBody,
    head: PaytmHead,
}

fn generate_paytm_signature(data: &str, key: &str) -> String {
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(key.as_bytes()).expect("HMAC can take key of any size");
    mac.update(data.as_bytes());
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let paytm_mid = "oZVrNr17626321194049";
    let paytm_key = "lXIWm4uU1BmOSdjx";

    let request_body = PaytmInitiateBody {
        requestType: "Payment".to_string(),
        mid: paytm_mid.to_string(),
        websiteName: "WEBSTAGING".to_string(),
        orderId: "TEST_ORDER_123".to_string(),
        callbackUrl: "http://localhost:4201/?checkout=success".to_string(),
        txnAmount: PaytmTxnAmount {
            value: "100.00".to_string(),
            currency: "INR".to_string(),
        },
        userInfo: PaytmUserInfo {
            custId: "TEST_CUST_123".to_string(),
        },
        channelId: "WEB".to_string(),
        industryTypeId: "Retail".to_string(),
    };

    let body_json = serde_json::to_string(&request_body)?;
    println!("Request Body JSON: {}", body_json);

    let signature = generate_paytm_signature(&body_json, &paytm_key);
    println!("Generated Signature: {}", signature);

    let initiate_request = PaytmInitiateRequest {
        body: request_body,
        head: PaytmHead { signature },
    };

    let client = reqwest::Client::new();
    let res = client
        .post("https://securegw-stage.paytm.in/theia/api/v1/initiateTransaction")
        .header("Content-Type", "application/json")
        .json(&initiate_request)
        .send()
        .await?;

    println!("Status: {}", res.status());
    let response_text = res.text().await?;
    println!("Response: {}", response_text);

    Ok(())
}