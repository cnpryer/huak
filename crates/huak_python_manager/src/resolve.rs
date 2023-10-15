use crate::{releases::Release, version::RequestedVersion};

pub(crate) fn get_release(
    _version: &RequestedVersion,
    _strategy: &Strategy,
) -> Option<Release<'static>> {
    todo!()
}

pub(crate) enum Strategy {
    Auto,
}
