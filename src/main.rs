#![allow(non_snake_case)]
#![allow(unused_macros)]
use clap::clap_app;
use serde::Deserialize;
use serde_json::{Number, Value};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::thread;
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;

// fn single_upload_url_2(val: &str) -> &str {
//     format!(
//         "https://{}.fooddx.cdmp.cloud/api/aia/v1/food/register-image/1/SG",
//         *(val.clone())
//     )
//     .as_str()
// }

// not using a function like above, as made more sense to use macros for code generation,
// as macros are a metaprogramming tool. Plus wanted to mess with macros :P
// They can be thought of equivalent to functions in our use case.
macro_rules! single_upload_url {
    ($val:expr) => {
        format!("https://{}", $val).as_str()
    };
}
macro_rules! single_result_url {
    ($val:expr, $imgId:expr ) => {
        format!("https://{}{}", $val, $imgId).as_str()
    };
}
macro_rules! report_url {
    ($val:expr, $imgId:expr ) => {
        format!("https://{}{}", $val, $imgId).as_str()
    };
}
macro_rules! batch_upload_url {
    ($val:expr) => {
        format!("https://{}", $val).as_str()
    };
}
macro_rules! batch_result_url {
    ($val:expr) => {
        format!("https://{}", $val).as_str()
    };
}

// We store the different environment types in a rust `enum`.
// Their contents (Sit, Uat, Prd) can be thought of as variables to which we can
// assign certain functionality to (like `as_str` done here)
// From the rust book, "Enums allow you to define a type by enumerating its possible variants."
// Rust’s enums are most similar to algebraic data types in functional languages, such as F#, OCaml, and Haskell.
// We additionaly make them "copy-able" and "clone-able" so we can share them between parallel tasks cheaply.
#[derive(Copy, Clone)]
enum EnvTypes {
    Sit,
    Uat,
    Prd,
    Wst,
}
impl EnvTypes {
    fn as_str(&self) -> &str {
        match self {
            EnvTypes::Sit => "sit",
            EnvTypes::Uat => "uat",
            EnvTypes::Prd => "prd",
            EnvTypes::Wst => "wst",
        }
    }
}

// this defines the structure of the response expected from the uploadUrl endpoint(s)
#[derive(Deserialize, Debug)]
struct GetImgUploadUrl {
    uploadUrl: String,
    imageId: String,
}

#[derive(Debug)]
struct Timeout {
    details: String,
}
impl fmt::Display for Timeout {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}
impl Error for Timeout {
    fn description(&self) -> &str {
        &self.details
    }
}
// this pre-loads the test image and includes the bytes in the the resultant binary.
// This means that anyone with the binary does not need to have this image on their computer.
// in more rust-y terms, it has a static lifetime which allows it to be used in any function and treated
// like a constant.
const IMG_BYTES: &[u8; 668034] = std::include_bytes!("images/test.jpg");

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // we use a macro here to cleanly define the structure of our CLI.
    let matches = clap_app!(myapp =>
        (version: "1.0")
        (about: "Run post deploy sanity checks")
        (@arg sit: -s --sit "run for SIT endpoints")
        (@arg wst: -w --wst "run for SIT(WST) endpoints")
        (@arg uat: -u --uat "run for UAT endpoints")
        (@arg prd: -p --prd "run for PRD endpoints")
    )
    .get_matches();

    // we then perform actions based on what matches we get from the CLI input
    let mut env = EnvTypes::Prd; // default environment
    if matches.is_present("sit") {
        env = EnvTypes::Sit;
    }
    if matches.is_present("uat") {
        env = EnvTypes::Uat;
    }
    if matches.is_present("prd") {
        env = EnvTypes::Prd;
    }
    if matches.is_present("wst") {
        env = EnvTypes::Wst;
    }
    println!("Running test for {} ⚡️", env.as_str());

    let mut tasks: Vec<JoinHandle<Result<(), reqwest::Error>>> = vec![];
    // timeout for requests
    let timeout = Duration::from_secs(3);

    // paralellize by executing the different aspects in seperate tasks
    tasks.push(tokio::spawn(async move {
        match get_endpoint_response(&env, timeout).await {
            Ok(_result) => {
                println!("\t✅ Successfully registered image and recieved response");
            }
            Err(e) => {
                println!(
                    "\t❌ Error while registering image and recieving response  : {}",
                    e
                );
            }
        }
        Ok(())
    }));
    tasks.push(tokio::spawn(async move {
                match get_report_response(&env, timeout).await {
                    Ok(_result) => {
                        println!("\t✅ Successfully reported image and had `underReview` be true on successive call");
                    }
                    Err(e) => {
                        println!(
                            "\t❌ Error while reporting image and marking `underReview` true  : {}",
                            e
                        );
                    }
                }
                Ok(())
            }));
    tasks.push(tokio::spawn(async move {
        match get_batch_imgs_response(3, &env, timeout).await {
            Ok(_result) => {
                println!("\t✅ Successfully registered a 3 image batch and recieved response");
            }
            Err(e) => {
                println!("\t❌ Error while registering 3 image batch : {}", e);
            }
        }
        Ok(())
    }));
    futures::future::join_all(tasks).await;
    Ok(())
}

async fn get_endpoint_response(env: &EnvTypes, timeout: Duration) -> Result<Value, Box<dyn Error>> {
    let client = reqwest::Client::new();
    // get upload url and response
    let imgUploadResp: GetImgUploadUrl = client
        .get(single_upload_url!(env.as_str()))
        .send()
        .await?
        .json::<GetImgUploadUrl>()
        .await?;
    // upload image contents to s3 using pre signed url
    let uploadResp = client
        .put(&imgUploadResp.uploadUrl)
        .header("content_type", "image/jpeg")
        .body(IMG_BYTES.to_vec())
        .send()
        .await?;
    assert!(true, uploadResp.status().is_success());
    // keep polling food rating endpoint every 200ms until food rating is ready.
    let start = Instant::now();
    loop {
        if start.elapsed() > timeout {
            return Err(Box::new(Timeout {
                details: format!("Request timed out after {:?}", timeout),
            }));
        }
        let img_result_resp = client
            .get(single_result_url!(env.as_str(), imgUploadResp.imageId))
            .send()
            .await?;
        if img_result_resp.status().is_success() {
            // use a generic response type as we don't really care about specifics here
            let jsonResp: Value = img_result_resp.json::<Value>().await?;
            return Ok(jsonResp);
        }
        thread::sleep(Duration::from_millis(200));
    }
}

async fn get_report_response(env: &EnvTypes, timeout: Duration) -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();
    // get upload url and response
    let imgUploadResp: GetImgUploadUrl = client
        .get(single_upload_url!(env.as_str()))
        .send()
        .await?
        .json::<GetImgUploadUrl>()
        .await?;
    // upload image to s3 using pre signed url
    let uploadResp = client
        .put(&imgUploadResp.uploadUrl)
        .header("content_type", "image/jpeg")
        .body(IMG_BYTES.to_vec())
        .send()
        .await?;
    assert!(true, uploadResp.status().is_success());
    // keep polling food rating endpoint every 200ms until food rating is ready.
    let start = Instant::now();
    loop {
        if start.elapsed() > timeout {
            return Err(Box::new(Timeout {
                details: format!(
                    "Request timed out after {:?}, while getting food rating the first time",
                    timeout
                ),
            }));
        }
        let img_result_resp = client
            .get(single_result_url!(env.as_str(), imgUploadResp.imageId))
            .send()
            .await?;
        if img_result_resp.status().is_success() {
            // use a generic response type as we don't really care about specifics here
            let _jsonResp: Value = img_result_resp.json::<Value>().await?;
            break;
        }
        thread::sleep(Duration::from_millis(200));
    }

    // do the actual reporting/flagging via another endpoint
    client
        .post(report_url!(env.as_str(), imgUploadResp.imageId))
        .send()
        .await?;
    // get the results again, same as above
    let start = Instant::now();
    loop {
        if start.elapsed() > timeout {
            return Err(Box::new(Timeout {
                details: format!(
                    "Request timed out after {:?}, while checking results again",
                    timeout
                ),
            }));
        }
        let img_result_resp = client
            .get(single_result_url!(env.as_str(), imgUploadResp.imageId))
            .send()
            .await?;
        if img_result_resp.status().is_success() {
            // use a generic response type as we don't really care about specifics here
            let jsonResp: Value = img_result_resp.json::<Value>().await?;
            // assert that underReview has been correctly marked as true
            assert_eq!(true, jsonResp["underReview"] == Value::Bool(true));
            break;
        }
        thread::sleep(Duration::from_millis(200));
    }
    Ok(())
}

async fn get_batch_imgs_response(
    num_times: usize,
    env: &EnvTypes,
    timeout: Duration,
) -> Result<serde_json::Value, Box<dyn Error>> {
    let mut header_map = HashMap::new();
    header_map.insert("imageCount", Value::Number(Number::from(num_times)));
    header_map.insert("entityId", Value::String("HMK-TEST-123".to_string()));
    header_map.insert("marketCode", Value::String("MY".to_string()));

    let client = reqwest::Client::new();
    let imgUploadResp = client
        .post(batch_upload_url!(env.as_str()))
        .json(&header_map)
        .send()
        .await?
        .json::<Vec<GetImgUploadUrl>>()
        .await?;
    // upload images
    for resp in &imgUploadResp {
        let uploadResp = client
            .put(&resp.uploadUrl)
            .header("content_type", "image/jpeg")
            .body(IMG_BYTES.to_vec())
            .send()
            .await?;

        assert!(true, uploadResp.status().is_success());
    }

    // get the batch result
    let image_ids: Vec<String> = imgUploadResp.into_iter().map(|val| val.imageId).collect();
    let start = Instant::now();
    loop {
        if start.elapsed() > timeout {
            return Err(Box::new(Timeout {
                details: format!(
                    "Request timed out after {:?}, while getting food rating ready for batch",
                    timeout
                ),
            }));
        }
        let img_result_resp = client
            .post(batch_result_url!(env.as_str()))
            .json(&image_ids)
            .send()
            .await?;

        // use a generic response type as we don't really care about specifics here
        let jsonResp: Value = img_result_resp.json::<Value>().await?;
        // we check for the presence of "NotReady" instead of the status code,
        // as the API still gives a success status code even when it's not ready for batch images.
        let json_str = jsonResp[0].as_str().unwrap_or("");
        if !json_str.contains(&"NotReady") {
            return Ok(jsonResp);
        }
        thread::sleep(Duration::from_millis(200));
    }
}
