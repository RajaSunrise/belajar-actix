#[cfg(test)]
mod tests {



    // Note: Database tests usually require a running DB or mocking.
    // For unit tests, we test pure logic or mock endpoints.

    #[test]
    fn test_jwt_generation() {
        let token = crate::auth::create_jwt("user1", "admin");
        assert!(token.is_ok());

        let claims = crate::auth::validate_jwt(&token.unwrap());
        assert!(claims.is_ok());
        assert_eq!(claims.unwrap().role, "admin");
    }

    #[test]
    fn test_sanitization() {
        let filename = "My Video.mp4";
        let clean = sanitize_filename::sanitize(filename);
        assert_eq!(clean, "My Video.mp4");
    }
}
