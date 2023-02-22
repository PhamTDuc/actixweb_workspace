use actix_web::{Error, error::ErrorUnauthorized};
use chrono::{Utc, Duration};
use jsonwebtoken::{Header, EncodingKey};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims{
    pub iss: String,
    pub scope: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
}

impl Default for Claims{
    fn default() -> Self {
        Self { 
            iss: "send-email-service@actixweb-workspace.iam.gserviceaccount.com".to_string(), 
            scope: "https://mail.google.com/ https://www.googleapis.com/auth/gmail.modify https://www.googleapis.com/auth/gmail.compose".to_string(),
            aud: "https://oauth2.googleapis.com/token".to_string(), 
            exp: (Utc::now() + Duration::hours(1)).timestamp(),
            iat: Utc::now().timestamp(), }
    }
}

impl Claims {
    pub fn create_jwt()->Result<String, Error>{
        let header = Header::new(jsonwebtoken::Algorithm::RS256);
        let claims = Claims::default();
        let encoding_key = EncodingKey::from_rsa_pem("-----BEGIN PRIVATE KEY-----\nMIIEuwIBADANBgkqhkiG9w0BAQEFAASCBKUwggShAgEAAoIBAQDPGueMP2ZgTUJ0\ndAAT2v7IzIfPXxx+/0C/Kl/wQ5PDQPBcoX9NGIGlpSE5kf8XLy0Vs0/XLAFMelk9\n7/guRvDAn9koTlF6tIKUhhFt8Y+DmuKja41WzkRKDRRiUOINb6yzPjkJ3XEU4iLg\nce667/MdLS4diR4PUD0kfDC5DIjfCg6yiJc2Tcs6Q4ijmY8SqgSAGi7V0KtIjJto\nrtsRguWQviMjlNNBezOgdhOiLFbJbr/z+M7rekdGzZsQFLOmj7rA5+gLGxL+5QSw\nYQhWR/5u4D8h83zb3mIpsC+hVJt31k8CF8pYoIDkw+tGroGmjxUpQDKG1uJhx1fo\nVxi+DcwtAgMBAAECgf9HjGdUEzEQjpAOS+m1f/QgyiYmiloC/aEQVBzj5cZuv8tw\nSsJBpabCe/w5BrMIMBnsy2vxSCNF9XwCnDT8gQWS+Mxf/kyMnnuJHadiYyt24E4i\nZZ+LNjHhaMFUbFjn/G9n+mS3dMfRbcY6AqqF+BtzjWLMs2qtwbTl/8WpxkzZ8Xmi\n6PFjEqT84m5QoAfRBteim+xmhBMgogXLlfx525OdirQMBfk0pv9fD48kvYmkZ8P8\nbb5Srp5klDpYJ5PFG5a9QJusx8wVY0Na5vubuQCXt/Vr68bhAJTUK5IZzvG7AzCd\nI6kMYPkuLprW6Ad75SR7AaM3o6kGWHDzoDpk9ZECgYEA6IJXvuh5r7Gzv3jWyFBB\nSsqrpaihtizkzn9cHc9ShdRxKflSCswsljpCvKbQBcBXSfujuGb/k8zomLq7JfFq\nIj3BsHoJaDs1HV2AVztQMJgpzy/Oz18PPHk/BqYTDxnPpwRg2UAmZ6WkKQ1NORjV\njjHQleabLNdXb6M9uMrtWDECgYEA5AeBvYgus+CECtteFUgp0R7HGNSU+HMjglKN\nliQrsex1EUP+o9Qr1Kae4uprY48NsGwTis20p0fOfQE1YgPH6mc/khQUCrWqEM7B\n96M37ysMHA9Aoh5vjoGCL+klPfKHWy5ilzC4m5PBjWm1XA4m0OhBR8I2fWgtrU2K\n2ejSsL0CgYEA1UFn1bRImDs3EHF3Hndty3mkgebvm7YFjtkF1lmn6RP3T3ZcJ4cp\nhgid2YZu2jeWWEcz3RirZhbVZ/AtYxnQLOPT2Ve1dSKJDwwJgPjoDgoinuPMnisk\nQGU8x45fgMQ9z4SWh+zszLgCn5yRrcL3bNqH5FWFElXY3o4tOGb31qECgYBg4kOg\niKz3JixUBJJ7zlZeEIqdJS1KRnlO1M5tfV7HUOUIefBGIU6iIk0dsmYAPfad/71p\n2y6naF32RL4ur9aP5GBhK4C8cCUa0Q4erk2Eo7CYd26Jsw7I6IOs4Y0+Pb5+4j1n\naX72MxgogkJZI3Ygip255G2MaOgHWwvkvARQrQKBgHjw08CvRREhsvh9aTIGnkW0\n6J7w3pEVlZ5+/U4r6KMwpApLVRa8KKDttvS/KaVuhfNAYptysn5DIW3Nrcst8rjy\nGXmC1iPxhAiTa8WQDdRx4ZtLprEaDFdbo0kgP3xxACarCop2zcASHvDqN5sOEo8T\ncGFxTIIW4AYU49j/xF4p\n-----END PRIVATE KEY-----\n".as_bytes()).expect("Fail to generate encoding key");
        return jsonwebtoken::encode(&header, &claims, &encoding_key).map_err(|e| ErrorUnauthorized(e.to_string()))
    }
    
}


