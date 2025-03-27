use protocol::keyring::keypair::KeyPairing;

pub trait DidAccessor {
    fn get_my_did(&self) -> String;
    fn get_my_keyring(&self) -> KeyPairing;
}

pub struct DidAccessorImpl {}

impl DidAccessor for DidAccessorImpl {
    fn get_my_did(&self) -> String {
        unimplemented!("get_my_did")
    }

    fn get_my_keyring(&self) -> KeyPairing {
        unimplemented!("get_my_keyring")
    }
}
