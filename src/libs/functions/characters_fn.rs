use bcrypt::{hash, verify, DEFAULT_COST};
use regex::Regex;

use itertools::Itertools;
use rand::seq::SliceRandom;
use rand::thread_rng;

pub fn is_valid_name(name: &str) -> Result<String, String> {
    // Define a regular expression that allows letters, spaces, and some special characters
    let re = Regex::new(r"^[a-zA-Z\s\-'\.,]*$").unwrap();

    // Check if the name matches the pattern
    if re.is_match(name) {
        // Check if it's a full name (at least two words)
        let word_count = name.split_whitespace().count();
        if word_count >= 2 {
            Ok(name.to_string()) // If valid and full name, return as is
        } else {
            Err(
                "Name is valid but not a full name. Please provide both first and last names."
                    .to_string(),
            )
        }
    } else {
        // Collect disallowed characters and create a corrected name
        let invalid_chars: Vec<char> = name
            .chars()
            .filter(|&c| {
                !Regex::new(r"[a-zA-Z\s\-'\.,]")
                    .unwrap()
                    .is_match(&c.to_string())
            })
            .collect();

        // Remove invalid characters to form a corrected name
        let corrected_name: String = name
            .chars()
            .filter(|&c| {
                Regex::new(r"[a-zA-Z\s\-'\.,]")
                    .unwrap()
                    .is_match(&c.to_string())
            })
            .collect();

        if !corrected_name.is_empty() {
            Err(format!(
                "contains disallowed characters [{}]. Suggested name: '{}'.",
                invalid_chars.iter().collect::<String>(),
                corrected_name
            ))
        } else {
            Err(format!(
                "contains disallowed characters [{}]. Please try another name.",
                invalid_chars.iter().collect::<String>()
            ))
        }
    }
}

pub fn generate_username(name: &str) -> String {
    let mut rng = thread_rng();

    // Split the name into words and convert to lowercase
    let binding = name.to_lowercase();
    let words: Vec<&str> = binding.split_whitespace().collect();

    // Generate all possible username combinations
    let mut possible_usernames = Vec::new();

    // Generate permutations and format them as potential usernames
    for permutation in words.iter().permutations(words.len()) {
        let joined = permutation.into_iter().join("_");
        possible_usernames.push(joined.clone());
        possible_usernames.push(format!("_{}", joined));
        possible_usernames.push(format!("{}_{}", joined, ""));
    }

    // Shuffle the usernames to introduce randomness
    possible_usernames.shuffle(&mut rng);

    let username = possible_usernames
        .first()
        .unwrap_or(&words.join("_"))
        .to_string();

    let username_with_suffix = format!("{}_{}", username, rand::random::<u8>());

    username_with_suffix
}

pub fn is_valid_username(username: &str) -> Result<String, String> {
    let re = Regex::new(r"^[a-zA-Z0-9_-]*$").unwrap();

    if re.is_match(username) {
        Ok(username.to_string())
    } else {
        let invalid_chars: Vec<char> = username
            .chars()
            .filter(|&c| !re.is_match(&c.to_string()))
            .collect();

        let corrected_username: String = username
            .chars()
            .filter(|&c| re.is_match(&c.to_string()))
            .collect();

        if !corrected_username.is_empty() {
            Err(format!(
                "Invalid username: contains disallowed characters [{}]. Suggested username: '{}'.",
                invalid_chars.iter().collect::<String>(),
                corrected_username
            ))
        } else {
            Err(format!(
                "Invalid username: contains disallowed characters [{}]. Please try another name.",
                invalid_chars.iter().collect::<String>()
            ))
        }
    }
}

pub fn validate_datetime(input: &str) -> Result<(), String> {
    let datetime_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d{1,3})?Z$").unwrap();

    if datetime_regex.is_match(input) {
        Ok(())
    } else {
        let mut error_message = String::new();

        // Check if the date part is valid
        if !Regex::new(r"^\d{4}-\d{2}-\d{2}").unwrap().is_match(input) {
            error_message.push_str("Invalid date format. The date should be in the format YYYY-MM-DD (e.g., 2024-11-15).\n");
        }

        // Check if the time part is valid
        if !Regex::new(r"T\d{2}:\d{2}:\d{2}").unwrap().is_match(input) {
            error_message.push_str("Invalid time format. The time should be in the format HH:MM:SS (e.g., 12:08:14).\n");
        }

        // Check if it ends with 'Z'
        if !input.ends_with('Z') {
            error_message.push_str("The string must end with 'Z' to indicate UTC timezone.\n");
        }

        // Suggest a valid format
        error_message.push_str(
            "\n Correct format: YYYY-MM-DDTHH:MM:SS.sssZ (e.g., 2024-11-15T12:08:14.128Z)\n",
        );
        Err(error_message)
    }
}

pub fn is_valid_email(email: &str) -> Result<String, String> {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();

    if email_regex.is_match(email) {
        Ok(email.to_string())
    } else {
        let invalid_chars: Vec<char> = email
            .chars()
            .filter(|&c| !email_regex.is_match(&c.to_string()))
            .collect();

        Err(format!(
            "Invalid email: contains disallowed characters [{}]. Please provide a valid email address.",
            invalid_chars.iter().collect::<String>()
        ))
    }
}

pub fn generate_code() -> String {
    let mut rng = thread_rng();
    let chars: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789".chars().collect();
    (0..5).map(|_| *chars.choose(&mut rng).unwrap()).collect()
}

pub fn hash_password(password: &str) -> String {
    hash(password, DEFAULT_COST).expect("Failed to hash password")
}

pub fn verify_password(hashed_password: &str, password: &str) -> bool {
    verify(password, hashed_password).unwrap_or(false)
}

pub fn is_date_string(date: &str) -> bool {
    let datetime_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$").unwrap();
    datetime_regex.is_match(date)
}
