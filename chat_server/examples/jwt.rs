use anyhow::Result;
use jwt_simple::prelude::*;

const JWT_DURATION: u64 = 60 * 60 * 24 * 7;
const JWT_ISS: &str = "chat_server";
const JWT_AUD: &str = "chat_web";

// openssl genpkey -algorithm ed25519 -out private.pem
// openssl pkey -in private.pem -pubout -out public.pem

fn main() -> Result<()> {
    let encoding_pem = include_str!("../fixtures/encoding.pem");
    let decoding_pem = include_str!("../fixtures/decoding.pem");

    let test = Test {
        name: "abc".to_string(),
    };
    let token = sign(encoding_pem, test.clone())?;
    let test2 = verify(decoding_pem, &token)?;

    assert_eq!(test, test2);
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Test {
    name: String,
}

fn sign(encoding_pem: &str, user: impl Into<Test>) -> Result<String, jwt_simple::Error> {
    let claims = Claims::with_custom_claims(user.into(), Duration::from_secs(JWT_DURATION));
    let claims = claims.with_issuer(JWT_ISS).with_audience(JWT_AUD);

    let ed25519 = Ed25519KeyPair::from_pem(encoding_pem)?;
    ed25519.sign(claims)
}

fn verify(decoding_pem: &str, token: &str) -> Result<Test, jwt_simple::Error> {
    let opts = VerificationOptions {
        allowed_issuers: Some(HashSet::from_strings(&[JWT_ISS])),
        allowed_audiences: Some(HashSet::from_strings(&[JWT_AUD])),
        ..Default::default()
    };

    let claims =
        Ed25519PublicKey::from_pem(decoding_pem)?.verify_token::<Test>(token, Some(opts))?;
    Ok(claims.custom)
}
