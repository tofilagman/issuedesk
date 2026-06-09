//! Int-backed domain enums. Stored as `smallint` in Postgres and (de)serialized
//! as integers in JSON — matching the `@JsonValue`/`@JsonCreator` convention used
//! across the other Outsource projects.

use serde_repr::{Deserialize_repr, Serialize_repr};

macro_rules! int_enum {
    ($name:ident { $($variant:ident = $val:expr),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize_repr, Deserialize_repr)]
        #[repr(i16)]
        pub enum $name {
            $($variant = $val),+
        }

        impl $name {
            pub fn from_i16(v: i16) -> Option<Self> {
                match v {
                    $($val => Some(Self::$variant),)+
                    _ => None,
                }
            }
            pub fn as_i16(self) -> i16 {
                self as i16
            }
        }
    };
}

int_enum!(Role { Member = 0, Admin = 1 });

int_enum!(IssueType { Bug = 0, Task = 1, Story = 2, Epic = 3 });

int_enum!(IssueStatus { Todo = 0, InProgress = 1, InReview = 2, Done = 3 });

int_enum!(Priority { Low = 0, Medium = 1, High = 2, Urgent = 3 });

int_enum!(ProjectRole { Member = 0, Lead = 1 });

// Activity log action codes (append-only audit trail).
int_enum!(ActivityAction {
    Created = 0,
    StatusChanged = 1,
    AssigneeChanged = 2,
    Commented = 3,
    LabelAdded = 4,
    LabelRemoved = 5,
    AttachmentAdded = 6,
    AttachmentRemoved = 7,
    PriorityChanged = 8,
    TypeChanged = 9,
    TitleChanged = 10,
    LinkAdded = 11,
    LinkRemoved = 12,
});
