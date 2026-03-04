use std::fmt;
#[derive(Debug, PartialEq)]

struct Category {
    primary: String,
    secondary: Option<String>,
}
impl Category {
    fn new(primary: &str, secondary: &str) -> Self {
        Self {
            primary: primary.to_string(),
            secondary: Some(secondary.to_string()),
        }
    }
    fn from_primary(primary: &str) -> Self {
        Self {
            primary: primary.to_string(),
            secondary: None,    
        }
    }
    fn from_str(s: &str) -> Category {
        let parts: Vec<&str> = s.split("/").collect();
        if parts.len() < 2 {
            Category {
                primary: parts[0].to_string(),
                secondary: None,
            }
        } else {
            Category {
                primary: parts[0].to_string(),
                secondary: Some(parts[1].to_string()),
            }
        }
    }
}
impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.secondary {
            Some(sec) => write!(f, "{}/{}", self.primary, sec),
            None => write!(f, "{}", self.primary),
        }
    }
}
fn main(){}

#[cfg(test)]
mod tests {
    use crate::Category;

    #[test]
    fn creating_a_category() {
        let category = Category::new("primary", "secondary");
        assert_eq!(category.primary, "primary");
        assert_eq!(category.secondary.unwrap(), "secondary");
    }   
    #[test]
    fn creating_a_category_from_primary() {
        let category = Category::from_primary("primary");
        assert_eq!(category.primary, "primary");
        assert_eq!(category.secondary, None);
    }
    #[test]
    fn creating_a_category_from_string_with_both() {
        let category = Category::from_str("primary/secondary");
        assert_eq!(category.primary, "primary");
        assert_eq!(category.secondary.unwrap(), "secondary");
    }
     #[test]
    fn creating_a_category_from_string_with_primary_only() {
        let category = Category::from_str("primary");
        assert_eq!(category.primary, "primary");
        assert_eq!(category.secondary, None);
    }
}
