use http::StatusCode;
use libdav::PropertyName;

use crate::caldav::CaldavError;

#[inline]
pub fn get_node_prop_by_name(node: roxmltree::Node, prop: PropertyName) -> Option<String> {
    get_node_by_name(node, prop).and_then(|node| node.text().map(str::to_string))
}

#[inline]
pub fn get_node_by_name<'a, 'b>(
    node: roxmltree::Node<'a, 'b>,
    prop: PropertyName<'_, '_>,
) -> Option<roxmltree::Node<'a, 'b>> {
    node.descendants().find(|node| node.tag_name() == prop)
}
/// Checks if the status code is success. If it is not, return it as an error.
#[inline]
pub fn check_status(status: StatusCode) -> Result<(), CaldavError> {
    if status.is_success() {
        Ok(())
    } else {
        Err(CaldavError::ErrorResponse(status))
    }
}
