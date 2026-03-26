// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::time::SystemTime;

use crate::generated::config::Config;
use pdk::hl::*;
use pdk::jwt::model::SigningAlgorithm;
use pdk::jwt::model::{JWTClaims, SigningKeyLength};
use pdk::jwt::JwtGenerator;
use serde_json::Value;

async fn request_filter(
    request_state: RequestState,
    config: &Config,
    generator: &JwtGenerator,
) -> Flow<()> {
    let _headers_state = request_state.into_headers_state().await;

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut claims: HashMap<String, Value> = HashMap::new();
    claims.insert("iss".to_string(), "test_policy".to_string().into());
    claims.insert(
        "scope".to_string(),
        ["READ".to_string(), "WRITE".to_string()].into(),
    );
    claims.insert("iat".to_string(), now.into());
    claims.insert("exp".to_string(), (now + 60).into());

    let mut headers: HashMap<String, Value> = HashMap::new();
    headers.insert("kid".to_string(), config.kid.clone().into());
    headers.insert("jku".to_string(), config.jku.clone().into());

    let Ok(jwt_claims) = JWTClaims::new(None, None, None, claims, headers) else {
        return Flow::Break(Response::new(503));
    };

    let Ok(jwt) = generator.jwt(jwt_claims) else {
        return Flow::Break(Response::new(503));
    };

    Flow::Break(Response::new(200).with_body(jwt))
}

#[entrypoint]
async fn configure(launcher: Launcher, Configuration(bytes): Configuration) -> Result<()> {
    let config: Config = serde_json::from_slice(&bytes).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {}",
            String::from_utf8_lossy(&bytes),
            err
        )
    })?;

    let generator = JwtGenerator::new(
        SigningAlgorithm::Rsa,
        SigningKeyLength::Len256,
        &config.private_key,
    )?;

    let filter = on_request(|rs| request_filter(rs, &config, &generator));

    launcher.launch(filter).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use pdk_unit::{UnitHttpMessage, UnitHttpRequest, UnitTestBuilder};
    use serde_json::json;

    const TEST_RSA_PRIVATE_KEY: &str = "-----BEGIN PRIVATE KEY-----
MIIJQQIBADANBgkqhkiG9w0BAQEFAASCCSswggknAgEAAoICAQC/OrFHGMNbsquO
eilUI39KUhCknuBlxf+3mXODrDp8iPm9/tQ/E1S2Jatp5ip9cGFv/FA7ndImgUMI
HZnRVkNaCk6Yl/+SA+0C1IIcBvR4Z7rroSXH4vQtBBTnPVyxEDOZrupp/N5Aatht
qgqCldERiPu6y/GSil0UhO/nFrlOZ74NA9BeAtQhRlENkTML9ObaxO/Dv5jzJ4Ik
IZJ8XKforeXq3CZBEm2g1EZEU2WD2vDZxv7WdP1z/Tx6+PLbnLBTy5E7ZUOx9bHo
aM+oXPCzrA2e5c9xhY6M1JP8ccl9qoOUhVUBMiXIRSR0OeasAz/dEC+qlx8YT8Fp
JCQMadR6NuogAWXk2SL1fxtmc0zKYWXJj7Diit3pBmUsXsmOxQGGV9uKeWJuz4aV
3bc0uMGSWcBGoVWOVQtw8dLNFWUdZYvr06hARcSLCRhLbP4CCQBgXe5MqKXDxa4X
wYymhFcxva/4XpenrEdxQeuTacDzPoJAnIqsEOHAl7POcWzJbHXm0PWnQ4GVY+4U
RTide+bW5YfhAv/FojAOG8+nM49laVZuuQ4PkKD5tzoLMVh5mIaUveiJG//cFXvM
GtGALgUNzlcotbG8GYXT2U1EL5znwAOrxHxWrEjBIrT13B+OwmaJ21fpcd8KIzla
s9g4SzAMYxOLbgxF0f2p+mFchc6tbwIDAQABAoICAB2z3KBZ8NI37NzLDctTXiyp
lYs0YE9+kyst6xrbMBRy5DPGNqp7cq9+J2NiDFyCjafqzX2NFHzFnCdRDbjNyNVd
/3pFNb203WYQowr+a4+eMRLza15iWqH5XdPTHKgmB5XJ7QA8djsUPXy/KjXBVoF+
QPdxQRsNYcrToT3IMk1C4Oq9mmpXzyJB/Un5sS+cwRTe/QzvIC84hkbdbhbh/3St
OiaiPlDiL2QJRMbNG1oBMmLpPWELN+kBvxisvXAuJNdHKc5LetnT+2fJi+OvV/XY
dh8lu/R6lbs7M6dE91KNHzX9BciTRPoX/0MMUU+Li6pnHrhFE9/fV3/gzLae45E5
CX2JfciE89W9cO/moPoh9jtfCeu1ojRY3/HcgeRBxppPmZoybUamoQYnC9LpisA2
3Zn+IsPSXHHznhc7gOhXvqLBgSdRs450hqhOyJ/hwXSeeoKbqtx81YIhZeauUuzZ
8oiYibHfFLJhWoH/sqSSF4D1tJQysybKD8Zw2dQz34JW3hsUXMu/mmcYd9almWJm
9toCtmV8pVL5MlbC635A1rfpnbtPaLLEtPoP1XAnHsquvy/yu6Njhj1zqbe2D3nH
5Q9EYikML5A6cBPzYcxtvmST3I58MyD3gPMxxEqYrF5GFNF+ty96MX9y+B4mYcGQ
Yv2sVyXOgQz4Wz16zMWNAoIBAQD1DlBYEcYsklVT/gIOjr2QlmonbCgtoE7rarr4
XXYFUL96IYlQD9BzKocka3Hu7YJtLf0SOzMZzmidc424DwWEMhIeuyPUrCFIgIYl
KzTxKTijwDpmMQQN2JGFP3R6zBSXyEr4XLUnOAu2FfxWSU5jVqaTKD9iB+bCm7+/
prg54EXPIr7Mh9+qkx4Vq1VZs7/5RnmZtDJ2+UbV0G8d9HbuRBz0IHvY1e7JZ8Nz
4G3VM5zETvxvFjgxg4EfcCDPt26yxRLJMazihsMWrtvMjvMUAhiil+Df+XOJMCXx
zrNDjn5M8A7pgilXlGzXqF7JLzB04cDDX/+CwqpW0QDSwHL1AoIBAQDHxPyqUFwe
ZB7Sgwv09cKaji3ez8F0as6Z3uY6odhaBLIJuei3XNO0DF39iYbgEXS96bHeja0f
5SsIdcJfryX1q+w820kCOyB0ZwL/JU86PDfJSn/nJ6bqrArr/R7RbytcN1wKuEcP
93TI9fXHyq2GyEnlO385G+3YZj6JU/d6ZBS0zsW/eCd2B1ze47O1Ti/mM2wGQy7M
vEoLqC5OwvOAisZZg8qWr0/Z31aR6V2lMo6dMdG8RrmAan5DKai1DG2EmFssAJIH
cAXB+EQ2MhyRrhk33nDgBf2mJT4ZLjszBZcutodw3pbzIM62489V0dUmOWKRgPPX
7d5YM2z88shTAoIBABp0cCIB0TYQmhuWKVyu9jH8uvsEhxXd34c0n3iehlYukG07
35oACw3TwoEhBEy54UGuHEryjyKzEMImrl73aC4MRb6Bj22vI2yzS0gJ8Q4z2AR9
hRBxLDHedl8/KXD0RSjZm5ZSU9AnEcSXfQVHpqm8ugDa8HTBy5youbuT4QGGf6LL
6nMkG/ZLKY1HUNB9QjVD8W6xcF09rfL5LHW8ZXZ1bfbA5v3SopOlmwkQamsAxmS+
7iuD548Y1kCxlyk1cULlWZDUxwgxajAxslLT/9PiIgyzfrhPMrTVuNLw8JNTd7kQ
lVuKDLKCuHlTmN/5My77DBdLbscMAt2adI9L7V0CggEAWQmpe9eZV0pUmosiFyo6
dFyOgVKj7Nl2AArjHproLScOm1srKB7NlOA2PDzByri9CbBRQNpwoVipF3o1CiSs
jJT2FCHApqfnzTnkkgf1CgWw75yu6T45HTtVGt2UkNA1yUI7WePMeIdYnAFUbJof
QYWfufYMvE2AcwUPNnIgSYK13+iRJsfM/sRFVmqyvEp++uFMcnYbM9FwR0XMbfpi
QZaY1WjyMLsuofLzSNF0lZ61BccgrgPvxhaw9AprUVaasZCegjw22e3KAyw+atFm
/l9UihwwvwishxLuXJbId/Mz8PQV5e6v5OloeQeMb7m4gPLuxd9tz34LrdAt8Yfc
VQKCAQBIi8Hq+efP5VeUkJ9aNplDk4mZsUx7bDZXu/x+oNncggJkO1rWSJf0MX+H
noC28uo0yL+OKo7H33XSfCmiTvXKw8EyAwSk5QoPIG3FnDI22IvzUyhRqrNKjA1a
LyitXE6ZcACrGQTvnT2J4pQqQICTVl7gczY5qZKnfhnE9q20jfgwnQyElU+qQtuw
GgHRzWyYVMDY+soTKyAPY161jvoWRgJOrfVC8wvb8xiMRVn1HJXTDuoK1MX35pGK
EjInQztINGCr1RfQeA/LKdDpqJREHaAfsKLJWIjkXFYnSnaeT4uG+GeiU3yQpNeS
AhW8CZzWMXVPOgExNZJBW65D7p/V
-----END PRIVATE KEY-----";

    fn config() -> String {
        json!({
            "privateKey": TEST_RSA_PRIVATE_KEY,
            "kid": "1",
            "jku": "https://test-jwks.example.com"
        })
        .to_string()
    }

    #[test]
    fn request_returns_200_with_jwt_body() {
        let mut tester = UnitTestBuilder::default()
            .with_config(config())
            .with_entrypoint(crate::configure);

        let response = tester.request_full(UnitHttpRequest::get());

        assert_eq!(response.status_code(), 200);
        assert!(!response.body().is_empty());
    }

    #[test]
    fn generated_jwt_has_three_parts() {
        let mut tester = UnitTestBuilder::default()
            .with_config(config())
            .with_entrypoint(crate::configure);

        let response = tester.request_full(UnitHttpRequest::get());

        let body = std::str::from_utf8(response.body()).unwrap();
        assert_eq!(body.split('.').count(), 3);
    }
}
