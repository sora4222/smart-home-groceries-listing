//! The name other members see on a dislike badge.
//!
//! The session token carries an id and, sometimes, an email — not a display
//! name. The part of the email before `@` is the friendliest name to hand;
//! without one the badge says "A household member" rather than a raw id.

/// What the badge calls a member when nothing better is known.
pub const UNKNOWN_MEMBER: &str = "A household member";

/// The name to show for the member with this email.
pub fn member_name(email: Option<&str>) -> String {
    email
        .and_then(|email| email.split('@').next())
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| UNKNOWN_MEMBER.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_part_before_the_at_sign_is_the_name() {
        assert_eq!(member_name(Some("phu@example.com")), "phu");
    }

    #[test]
    fn no_email_means_a_household_member() {
        assert_eq!(member_name(None), UNKNOWN_MEMBER);
    }

    #[test]
    fn an_email_with_nothing_before_the_at_sign_means_a_household_member() {
        assert_eq!(member_name(Some("@example.com")), UNKNOWN_MEMBER);
        assert_eq!(member_name(Some("  ")), UNKNOWN_MEMBER);
    }

    #[test]
    fn an_email_without_an_at_sign_is_used_whole() {
        assert_eq!(member_name(Some("jesse")), "jesse");
    }
}
