/// OwnerReference contains enough information to let you identify an owning object. An owning object must be in the same namespace as the dependent, or be cluster-scoped, so there is no namespace field.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OwnerReference {
    /// API version of the referent.
    pub api_version: String,

    /// If true, AND if the owner has the "foregroundDeletion" finalizer, then the owner cannot be deleted from the key-value store until this reference is removed. See https://kubernetes.io/docs/concepts/architecture/garbage-collection/#foreground-deletion for how the garbage collector interacts with this field and enforces the foreground deletion. Defaults to false. To set this field, a user needs "delete" permission of the owner, otherwise 422 (Unprocessable Entity) will be returned.
    pub block_owner_deletion: Option<bool>,

    /// If true, this reference points to the managing controller.
    pub controller: Option<bool>,

    /// Kind of the referent. More info: https://git.k8s.io/community/contributors/devel/sig-architecture/api-conventions.md#types-kinds
    pub kind: String,

    /// Name of the referent. More info: https://kubernetes.io/docs/concepts/overview/working-with-objects/names#names
    pub name: String,

    /// UID of the referent. More info: https://kubernetes.io/docs/concepts/overview/working-with-objects/names#uids
    pub uid: String,
}
