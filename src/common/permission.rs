use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Default)]
pub enum Role {
    Owner,
    Organizer,
    FileOrganizer,
    Writer,
    Commenter,
    #[default]
    Reader,
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Role::Owner => write!(f, "owner"),
            Role::Organizer => write!(f, "organizer"),
            Role::FileOrganizer => write!(f, "fileOrganizer"),
            Role::Writer => write!(f, "writer"),
            Role::Commenter => write!(f, "commenter"),
            Role::Reader => write!(f, "reader"),
        }
    }
}

impl FromStr for Role {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "owner" => Ok(Role::Owner),
            "organizer" => Ok(Role::Organizer),
            "fileOrganizer" => Ok(Role::FileOrganizer),
            "writer" => Ok(Role::Writer),
            "commenter" => Ok(Role::Commenter),
            "reader" => Ok(Role::Reader),
            _ => Err(format!("'{}' is not a valid role, valid roles are: owner, organizer, fileOrganizer, writer, commenter, reader", s)),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum Type {
    User,
    Group,
    Domain,
    #[default]
    Anyone,
}

impl Type {
    pub fn requires_email(&self) -> bool {
        match self {
            Type::User => true,
            Type::Group => true,
            Type::Domain => false,
            Type::Anyone => false,
        }
    }

    pub fn requires_domain(&self) -> bool {
        match self {
            Type::User => false,
            Type::Group => false,
            Type::Domain => true,
            Type::Anyone => false,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::User => write!(f, "user"),
            Type::Group => write!(f, "group"),
            Type::Domain => write!(f, "domain"),
            Type::Anyone => write!(f, "anyone"),
        }
    }
}

impl FromStr for Type {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user" => Ok(Type::User),
            "group" => Ok(Type::Group),
            "domain" => Ok(Type::Domain),
            "anyone" => Ok(Type::Anyone),
            _ => Err(format!(
                "'{}' is not a valid type, valid types are: user, group, domain, anyone",
                s
            )),
        }
    }
}
