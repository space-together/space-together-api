use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};

const ARGON2_MEMORY_COST_KIB: u32 = 4_096;
const ARGON2_TIME_COST: u32 = 2;
const ARGON2_PARALLELISM: u32 = 1;

fn password_hasher() -> Argon2<'static> {
    let params = Params::new(
        ARGON2_MEMORY_COST_KIB,
        ARGON2_TIME_COST,
        ARGON2_PARALLELISM,
        None,
    )
    .expect("valid Argon2 parameters");

    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
}

pub fn hash_password(password: &str) -> String {
    let salt = SaltString::generate(&mut OsRng); // secure random salt
    let argon2 = password_hasher();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

pub fn verify_password(hash: &str, password: &str) -> bool {
    let Ok(parsed_hash) = argon2::password_hash::PasswordHash::new(hash) else {
        return false;
    };
    let argon2 = password_hasher();

    argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

pub fn password_needs_rehash(hash: &str) -> bool {
    !hash.contains("$m=4096,t=2,p=1$")
}
