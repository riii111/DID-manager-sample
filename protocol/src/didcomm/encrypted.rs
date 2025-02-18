use crate::did::did_repository::DidRepository;
use crate::verifiable_credentials::did_vc::DidVcService;

pub struct DidCommServiceWithAttachment<R>
where
    R: DidRepository + DidVcService,
{
    vc_service: R,
    attachment_link: String,
}

impl<R> DidCommServiceWithAttachment<R>
where
    R: DidRepository + DidVcService,
{
    pub fn new(did_repository: R, attachment_link: String) -> Self {
        Self {
            vc_service: did_repository,
            attachment_link,
        }
    }
}
