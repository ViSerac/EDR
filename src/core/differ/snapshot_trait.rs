pub trait Snapshot {
    type Identity;
    type Item;

    fn identities(&self) -> Vec<Self::Identity>;
    fn get(&self, identity: &Self::Identity) -> Option<&Self::Item>;
}