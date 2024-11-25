use regex::Regex;

use itertools::Itertools; // Import permutations
use rand::seq::SliceRandom; // Import random shuffling
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
