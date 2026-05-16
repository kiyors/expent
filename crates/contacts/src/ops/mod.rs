pub mod create;
pub mod delete;
pub mod get;
pub mod identifiers;
pub mod list;
pub mod merge;
pub mod resolve;
pub mod suggestions;
pub mod update;

pub use create::create_contact;
pub use delete::delete_contact;
pub use get::get_contact_detail;
pub use identifiers::add_contact_identifier;
pub use list::list_contacts;
pub use merge::merge_contacts;
pub use resolve::{ContactResolution, ResolveParams, resolve_contact, resolve_contacts_bulk};
pub use suggestions::{MergeSuggestion, get_merge_suggestions};
pub use update::update_contact;
